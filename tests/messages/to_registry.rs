//! What a station asks its registry, and what the registry answers.

use ankurah::proto::{Attested, Event, Membership, ModelId, Operation, OperationSet};
use koinon_commons::{catalog::names_driver, codec::RecordBuilder};

use super::{
    checks::{place, Place},
    *,
};

fn for_application() -> ForApplication {
    ForApplication { key: key_of(&application()) }
}

/// A registration under the grant whose record is `grant`, written at `registered`.
fn registration(grant: u8, registered: i64) -> Registration {
    Registration { grant: entity(grant).into(), registered: at(registered) }
}

/// A record of the hello's answer for one registration.
fn registered(registration: &Registration) -> Result<RecordBuilder, Box<dyn Error>> {
    let mut record = RecordBuilder::new(None);
    record.add_must_understand(registration)?.add(&for_application())?;
    Ok(record)
}

/// What a registry's answer to a hello says, as the station reads it: each registration it keeps
/// for the station, with its application; each retirement the station missed, as its key signed
/// it; and each voiding of an application's slips, with its application.
#[derive(Debug, Default, PartialEq)]
struct HelloAnswer {
    registrations: Vec<(Registration, ForApplication)>,
    retirements: Vec<SignedRetirement>,
    voidings: Vec<(SlipsVoided, ForApplication)>,
}

/// The answer to a hello, read as the station reads it, which tells a refusal from the answer
/// first, since an answer with no record says that the registry keeps no registration for the
/// station: each record, under no entity id, a registration, a retirement or a voiding. A record
/// of news this station does not know is read past.
fn hello_answer(answer: &[u8]) -> Result<HelloAnswer, Box<dyn Error>> {
    let read = reply(answer, |answer| {
        let mut hello = HelloAnswer::default();
        for record in answer.records() {
            let components = record.components();
            if record.entity().is_some() {
                return Err("a record of a hello's answer names no entity".into());
            } else if components.has::<Registration>() {
                hello.registrations.push(components.read()?);
            } else if components.has::<SignedRetirement>() {
                hello.retirements.push(components.read()?);
            } else if components.has::<SlipsVoided>() {
                hello.voidings.push(components.read()?);
            }
        }
        Ok(hello)
    })?;
    Ok(read.map_err(|refused| format!("the hello was refused as {}", refused.refusal.code))?)
}

/// A station that has just connected learns which registrations the registry keeps for it, each
/// under its grant, and the news it missed: a retirement it must apply, and a voiding of slips,
/// which it may not drop. An answer with no record says the registry keeps none, and a refusal of
/// the hello does not read as one.
#[test]
fn a_hello_is_answered_with_the_registrations_and_the_news_the_station_missed() -> Outcome {
    let request = EnvelopeBuilder::new().add_must_understand(&Hello {})?.encode();
    let received = registry_reads(&request)?;
    assert_eq!(received.must_understand(), [Hello::ID]);
    received.components().read::<Hello>()?;

    let kept = registration(40, REGISTERED);
    let retirement: SignedRetirement = signed(EnvelopeBuilder::new().add_must_understand(&Retirement { retired: at(NOW - 5 * MINUTE) })?);
    let voided = SlipsVoided { before: at(NOW - MINUTE) };
    let mut news = RecordBuilder::new(None);
    news.add_must_understand(&retirement)?;
    let mut voiding = RecordBuilder::new(None);
    voiding.add_must_understand(&voided)?.add(&for_application())?;
    let mut answer = EnvelopeBuilder::new();
    answer.add_record(registered(&kept)?)?.add_record(news)?.add_record(voiding)?;
    let answer = answer.encode();

    let hello = hello_answer(&answer)?;
    assert_eq!((hello.registrations, hello.voidings), (vec![(kept, for_application())], vec![(voided, for_application())]));
    let [read_retirement] = &hello.retirements[..] else { return Err("the answer carries one retirement".into()) };
    assert_eq!(read_retirement.signer, for_application().key);
    let retired = read_retirement.open(&Limits::default(), RETIREMENT_UNDERSTANDS)?;
    assert_eq!(retired.components().read::<Retirement>()?, Retirement { retired: at(NOW - 5 * MINUTE) });

    assert_eq!(hello_answer(&EnvelopeBuilder::new().encode())?, HelloAnswer::default());
    let refusal = Refusal { code: Refusal::INTERNAL_FAILURE.into(), message: "the registry's store did not answer".into() };
    assert!(hello_answer(&EnvelopeBuilder::new().add(&refusal)?.encode()).is_err());
    Ok(())
}

/// A person's sign-in, answered with who signed in and the places they may register the
/// station: here a sub-scope alone, under its scope, which comes without `Registrable`.
#[test]
fn a_sign_in_is_answered_with_the_person_and_the_scopes_they_may_register_in() -> Outcome {
    let sign_in = IdTokenSignIn {
        id_token: "eyJhbGciOiJSUzI1NiJ9.eyJzdWIiOiIxIn0.c2ln".into(),
        nonce_seed: "kV3pQ8sN1xR6tB0yW4zH7mC2fJ9dL5gA8eU3oI1nT6c".into(),
    };
    let request = EnvelopeBuilder::new().add_must_understand(&SignIn {})?.add(&sign_in)?.encode();
    let received = registry_reads(&request)?;
    assert_eq!(received.components().read::<(SignIn, IdTokenSignIn)>()?.1, sign_in);

    let person = SignedInPerson { name: Some("Ada Lovelace".into()), email: None };
    let (acme, east) = (entity(20), entity(21));
    let scope = Scope { name: "Acme".into(), parent: None };
    let sub_scope = Scope { name: "East".into(), parent: Some(acme.into()) };
    let mut whole = RecordBuilder::new(Some(acme));
    whole.add(&scope)?;
    let mut registrable = RecordBuilder::new(Some(east));
    registrable.add(&sub_scope)?.add(&Registrable {})?;
    let mut answer = EnvelopeBuilder::new();
    answer.add(&person)?.add_record(whole)?.add_record(registrable)?;
    let answer = answer.encode();

    let read = reply(&answer, |answer| {
        let scopes = answer
            .records()
            .map(|record| {
                record.components().read::<(Scope, Option<Registrable>)>().map(|(scope, registrable)| (record.entity(), scope, registrable))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok((answer.components().read::<SignedInPerson>()?, scopes))
    })?;
    assert_eq!(read, Ok((person, vec![(Some(acme), scope, None), (Some(east), sub_scope, Some(Registrable {}))])));
    Ok(())
}

/// A registration for one application, in a sub-scope the sign-in's answer listed, and one in a
/// first scope the person names; the application listed so that no registry takes either for one
/// for all applications.
#[test]
fn a_registration_names_its_station_its_place_and_its_application() -> Outcome {
    let assignment = ScopeAssignment { station: station_record().into() };
    let placement = Placement { scope: entity(20).into(), sub_scope: Some(entity(21).into()) };
    let first = FirstScope { name: Name::new("Acme")? };
    let listed = EnvelopeBuilder::new()
        .add_must_understand(&Register {})?
        .add(&assignment)?
        .add(&placement)?
        .add_must_understand(&for_application())?
        .encode();
    let in_a_first_scope = EnvelopeBuilder::new()
        .add_must_understand(&Register {})?
        .add(&assignment)?
        .add(&first)?
        .add_must_understand(&for_application())?
        .encode();

    let received = registry_reads(&listed)?;
    let mut must_understand = vec![Register::ID, ForApplication::ID];
    must_understand.sort();
    assert_eq!(received.must_understand(), must_understand);
    let (_, read_assignment, application) = received.components().read::<(Register, ScopeAssignment, ForApplication)>()?;
    assert_eq!(
        (read_assignment, place(&received.components())?, application),
        (assignment.clone(), Place::Listed(placement), for_application())
    );

    let received = registry_reads(&in_a_first_scope)?;
    let (_, read_assignment, application) = received.components().read::<(Register, ScopeAssignment, ForApplication)>()?;
    assert_eq!((read_assignment, place(&received.components())?, application), (assignment, Place::First(first), for_application()));
    Ok(())
}

/// What a registration asks a registry: the person whose sign-in it accepted on the connection,
/// the station's record, and the place.
#[derive(Debug, PartialEq)]
struct Asked {
    person: &'static str,
    station: ScopeAssignment,
    place: Place,
}

/// A grant a registry wrote for a station's registration for the application: the key of the
/// station it is for, as the grant's `NodeGrant` holds it, the registration it answered with, what
/// was asked, and whether the grant is still open.
struct Written {
    station: PublicKey,
    registration: Registration,
    asked: Asked,
    open: bool,
}

/// What a registry keeps of stations' registrations for the application, each under the grant it
/// wrote for it, with the clock that gives each grant its moment.
#[derive(Default)]
struct Grants {
    written: Vec<Written>,
    clock: RegistryClock,
}

impl Grants {
    /// Answers `request`, a registration for the application of the station whose key the
    /// connection proves, `connection`, by `person`, whose sign-in the registry accepted on that
    /// connection, when the registry's clock reads `now`: a repeat of the registration that holds
    /// the station registered, by the same person, for the same station record and in the same
    /// place, with its `Registration` again; any other while that registration is open with a
    /// refusal as `not_authorized`; and one while none is open for the station with a
    /// `Registration` under a grant written anew for its key.
    fn register(&mut self, request: &[u8], person: &'static str, connection: &PublicKey, now: i64) -> Result<Vec<u8>, Box<dyn Error>> {
        let received = registry_reads(request)?;
        let (_, station, application) = received.components().read::<(Register, ScopeAssignment, ForApplication)>()?;
        if application != for_application() {
            return Err("the registry hosts no other application".into());
        }
        let asked = Asked { person, station, place: place(&received.components())? };
        let registration = match self.written.iter().find(|written| written.open && written.station == *connection) {
            Some(open) if open.asked == asked => open.registration.clone(),
            Some(_) => {
                let refusal =
                    Refusal { code: Refusal::NOT_AUTHORIZED.into(), message: "the station is registered for the application".into() };
                return Ok(EnvelopeBuilder::new().add(&refusal)?.encode());
            }
            None => {
                let grant = 40 + u8::try_from(self.written.len())?;
                let anew = registration(grant, self.clock.moment(now));
                self.written.push(Written { station: *connection, registration: anew.clone(), asked, open: true });
                anew
            }
        };
        Ok(EnvelopeBuilder::new().add(&for_application())?.add_must_understand(&registration)?.encode())
    }

    /// Ends the registration a notice that a station no longer shares names, by its grant alone,
    /// if the grant is still open and is for the application and for the station whose key the
    /// connection that brought the notice proves, `connection`: so that a notice under an old grant
    /// leaves a newer one open, and one from another station, which may know the grant's id, leaves
    /// the grant open. It is answered with `Done` either way.
    fn stop_sharing(&mut self, notice: &[u8], connection: &PublicKey) -> Result<Vec<u8>, Box<dyn Error>> {
        let received = registry_reads(notice)?;
        let (_, application, named) = received.components().read::<(StopSharing, ForApplication, ForGrant)>()?;
        for written in &mut self.written {
            if application == for_application() && written.station == *connection && written.registration.grant == named.grant {
                written.open = false;
            }
        }
        Ok(done())
    }

    fn open(&self) -> Vec<Registration> {
        self.written.iter().filter(|written| written.open).map(|written| written.registration.clone()).collect()
    }
}

/// The key of a second station, registered with the same registry for the same application.
fn other_station() -> PublicKey {
    key(10)
}

/// The second station's own record.
fn other_station_record() -> EntityId {
    entity(15)
}

/// A registration of the station whose record is `station`, for the application, in the sub-scope
/// whose record is `sub_scope`, under the scope the sign-in's answer listed.
fn registering(station: EntityId, sub_scope: u8) -> Result<Vec<u8>, Box<dyn Error>> {
    Ok(EnvelopeBuilder::new()
        .add_must_understand(&Register {})?
        .add(&ScopeAssignment { station: station.into() })?
        .add(&Placement { scope: entity(20).into(), sub_scope: Some(entity(sub_scope).into()) })?
        .add_must_understand(&for_application())?
        .encode())
}

/// A registration of the station for the application in the sub-scope whose record is
/// `sub_scope`.
fn registering_in(sub_scope: u8) -> Result<Vec<u8>, Box<dyn Error>> {
    registering(station_record(), sub_scope)
}

/// The registration a station keeps from the registry's answer to its `Register`, read as the
/// station reads a reply: an answer that is a refusal registers nothing.
fn kept(answer: &[u8]) -> Result<Registration, Box<dyn Error>> {
    let read = reply(answer, |answer| Ok(answer.components().read::<(ForApplication, Registration)>()?))?;
    let (application, registration) = read.map_err(|refused| format!("the registration was refused as {}", refused.refusal.code))?;
    if application != for_application() {
        return Err("the answer names another application".into());
    }
    Ok(registration)
}

/// The notice that the station no longer shares with the application under `registration`, which
/// names the registration's grant and nothing else of it.
fn notice(registration: &Registration) -> Result<Vec<u8>, Box<dyn Error>> {
    Ok(EnvelopeBuilder::new()
        .add_must_understand(&StopSharing {})?
        .add(&for_application())?
        .add_must_understand(&ForGrant { grant: registration.grant })?
        .encode())
}

/// A station whose answer was lost repeats its registration as it was, and is answered with the
/// same registration, under the grant already written and with its moment; while that
/// registration is open, any other registration of the station for the application, in another
/// place or by another person, is refused as `not_authorized`.
#[test]
fn a_repeated_registration_is_answered_with_the_same_registration_and_any_other_is_refused() -> Outcome {
    let mut grants = Grants::default();
    let request = registering_in(21)?;
    let first = kept(&grants.register(&request, "the person who said yes", &station(), NOW - 10 * MINUTE)?)?;
    assert_eq!(kept(&grants.register(&request, "the person who said yes", &station(), NOW - 9 * MINUTE)?)?, first);
    let elsewhere = grants.register(&registering_in(22)?, "the person who said yes", &station(), NOW - 8 * MINUTE)?;
    assert_eq!(refusal_code(&elsewhere)?, Refusal::NOT_AUTHORIZED);
    let by_someone_else = grants.register(&request, "another person", &station(), NOW - 8 * MINUTE)?;
    assert_eq!(refusal_code(&by_someone_else)?, Refusal::NOT_AUTHORIZED);
    assert_eq!(grants.open(), [first]);
    Ok(())
}

/// A person removes the application and adds it again; the station's first notice, repeated after
/// its answer was lost, arrives after the new registration, and leaves that registration open,
/// since it names the grant it was about.
#[test]
fn a_notice_that_arrives_late_ends_its_own_registration_and_leaves_a_newer_one_open() -> Outcome {
    let mut grants = Grants::default();
    let request = registering_in(21)?;
    let first = kept(&grants.register(&request, "the person who said yes", &station(), NOW - 10 * MINUTE)?)?;
    let removal = notice(&first)?;
    assert_done(&grants.stop_sharing(&removal, &station())?)?;
    let second = kept(&grants.register(&request, "the person who said yes", &station(), NOW - 5 * MINUTE)?)?;
    assert_ne!(first.grant, second.grant, "a registration after a removal is under a new grant");

    assert_done(&grants.stop_sharing(&removal, &station())?)?;
    assert_eq!(grants.open(), std::slice::from_ref(&second));

    let mut hello = EnvelopeBuilder::new();
    for open in grants.open() {
        hello.add_record(registered(&open)?)?;
    }
    assert_eq!(hello_answer(&hello.encode())?.registrations, [(second, for_application())]);
    Ok(())
}

/// Two stations are registered for the same application, each under a grant for its own key. A
/// notice that names the first station's grant ends that registration only when it comes down the
/// first station's connection: from the second station, which may have learnt the grant's id, it
/// is answered with `Done` and leaves both grants open.
#[test]
fn a_notice_ends_a_registration_only_down_the_connection_of_the_station_its_grant_is_for() -> Outcome {
    let mut grants = Grants::default();
    let first = kept(&grants.register(&registering_in(21)?, "the person who said yes", &station(), NOW - 10 * MINUTE)?)?;
    let other = registering(other_station_record(), 21)?;
    let second = kept(&grants.register(&other, "the person who said yes", &other_station(), NOW - 9 * MINUTE)?)?;
    assert_ne!(first.grant, second.grant, "each station is registered under a grant of its own");

    let removal = notice(&first)?;
    assert_done(&grants.stop_sharing(&removal, &other_station())?)?;
    assert_eq!(grants.open(), [first.clone(), second.clone()]);
    assert_done(&grants.stop_sharing(&removal, &station())?)?;
    assert_eq!(grants.open(), [second]);
    Ok(())
}

/// The events of the station's own records reach the registry as Ankurah encodes them, each with
/// the id it had, and the registry answers that it imported them.
#[test]
fn a_push_hands_over_the_stations_events_unchanged() -> Outcome {
    // The genesis of the station's own record, as a member of the `Station` model.
    let mut operations = OperationSet::default();
    operations.push(Operation::Membership(Membership::Add(ModelId::EntityId(EntityId::from_bytes(*Station::ID.as_bytes())))));
    let pushed: Vec<Attested<Event>> = vec![Attested::from(Event::genesis_from_parts([7; 32], NOW as u64, operations))];
    let request = EnvelopeBuilder::new().add_must_understand(&PushEvents {})?.add(&Events::new(&pushed))?.encode();

    let received = registry_reads(&request)?;
    let (_, events) = received.components().read::<(PushEvents, Events)>()?;
    let imported = events.read()?;
    assert_eq!(imported, pushed);
    assert_eq!(imported[0].payload.id(), pushed[0].payload.id());
    assert_done(&done())
}

/// The catalog comes as records: each stock as the `Media` a person's pick is copied into, then
/// each printer fingerprint in the registry's order, by which the first that names a driver
/// decides.
#[test]
fn a_catalog_is_answered_with_a_record_for_each_stock_and_each_fingerprint_in_order() -> Outcome {
    let request = EnvelopeBuilder::new().add_must_understand(&GetCatalog {})?.encode();
    registry_reads(&request)?.components().read::<GetCatalog>()?;

    let stocks = [stock("4x6", 101.6, Some(152.4)), stock("80mm Continuous", 80.0, None)];
    let formats = |list: &[&str]| MimeTypes::new(list.iter().map(|format| MimeType::new(*format).unwrap()).collect()).unwrap();
    let fingerprints = [
        PrinterFingerprint { driver: "TLP 2824 Plus".into(), document_formats: formats(&["application/x-zpl2", "application/x-epl2"]) },
        PrinterFingerprint { driver: "TLP 2824".into(), document_formats: formats(&["application/x-epl2"]) },
    ];
    let mut answer = EnvelopeBuilder::new();
    for stock in &stocks {
        let mut record = RecordBuilder::new(None);
        record.add(stock)?;
        answer.add_record(record)?;
    }
    for fingerprint in &fingerprints {
        let mut record = RecordBuilder::new(None);
        record.add(fingerprint)?;
        answer.add_record(record)?;
    }
    let answer = answer.encode();

    let read: Vec<(Option<Media>, Option<PrinterFingerprint>)> =
        reply(&answer, |catalog| Ok(catalog.records().map(|record| record.components().read()).collect::<Result<_, _>>()?))?
            .map_err(|refused| format!("the catalog was refused as {}", refused.refusal.code))?;
    let read_stocks: Vec<Media> = read.iter().filter_map(|(stock, _)| stock.clone()).collect();
    assert_eq!(read_stocks, stocks);
    let read_fingerprints: Vec<PrinterFingerprint> = read.into_iter().filter_map(|(_, fingerprint)| fingerprint).collect();
    assert_eq!(read_fingerprints, fingerprints);
    let deciding = read_fingerprints.iter().find(|fingerprint| names_driver(&fingerprint.driver, "Zebra TLP2824 Plus")).unwrap();
    assert_eq!(deciding.document_formats.types().len(), 2, "the longer name, which stands first, decides");
    Ok(())
}

/// What Pylon 0.1.0's stations ask, a registration with no application among it, takes the same
/// components, so that the new protocol loses nothing the old one says.
#[test]
fn pylon_0_1_0s_registration_takes_the_same_components_without_an_application() -> Outcome {
    let assignment = ScopeAssignment { station: station_record().into() };
    let first = FirstScope { name: Name::new("Acme")? };
    let request = EnvelopeBuilder::new().add_must_understand(&Register {})?.add(&assignment)?.add(&first)?.encode();
    let received = registry_reads(&request)?;
    let (_, read_assignment, read_first, application) =
        received.components().read::<(Register, ScopeAssignment, FirstScope, Option<ForApplication>)>()?;
    assert_eq!((read_assignment, read_first, application), (assignment, first, None));
    Ok(())
}
