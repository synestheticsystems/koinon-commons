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

/// A station that has just connected learns which registrations the registry keeps for it, each
/// under its grant, and the news it missed: a retirement it must apply, and a voiding of slips,
/// which it may not drop.
#[test]
fn a_hello_is_answered_with_the_registrations_and_the_news_the_station_missed() -> Outcome {
    let request = EnvelopeBuilder::new().add_must_understand(&Hello {})?.encode();
    let received = registry_reads(&request);
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

    let received = decoded(&answer, REPLIES_UNDERSTOOD);
    let records: Vec<_> = received.records().collect();
    assert_eq!(records.len(), 3);
    assert!(records.iter().all(|record| record.entity().is_none()));
    assert_eq!(records[0].components().read::<(Registration, ForApplication)>()?, (kept, for_application()));
    let read_retirement: SignedRetirement = records[1].components().read()?;
    assert_eq!(read_retirement.signer, for_application().key);
    let retired = read_retirement.open(&Limits::default(), RETIREMENT_UNDERSTANDS)?;
    assert_eq!(retired.components().read::<Retirement>()?, Retirement { retired: at(NOW - 5 * MINUTE) });
    assert_eq!(records[2].components().read::<(SlipsVoided, ForApplication)>()?, (voided, for_application()));
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
    let received = registry_reads(&request);
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

    let received = decoded(&answer, REPLIES_UNDERSTOOD);
    assert_eq!(received.components().read::<SignedInPerson>()?, person);
    let read: Vec<(Option<EntityId>, Scope, Option<Registrable>)> = received
        .records()
        .map(|record| {
            record.components().read::<(Scope, Option<Registrable>)>().map(|(scope, registrable)| (record.entity(), scope, registrable))
        })
        .collect::<Result<_, _>>()?;
    assert_eq!(read, [(Some(acme), scope, None), (Some(east), sub_scope, Some(Registrable {}))]);
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

    let received = registry_reads(&listed);
    let mut must_understand = vec![Register::ID, ForApplication::ID];
    must_understand.sort();
    assert_eq!(received.must_understand(), must_understand);
    let (_, read_assignment, application) = received.components().read::<(Register, ScopeAssignment, ForApplication)>()?;
    assert_eq!(
        (read_assignment, place(&received.components())?, application),
        (assignment.clone(), Place::Listed(placement), for_application())
    );

    let received = registry_reads(&in_a_first_scope);
    let (_, read_assignment, application) = received.components().read::<(Register, ScopeAssignment, ForApplication)>()?;
    assert_eq!((read_assignment, place(&received.components())?, application), (assignment, Place::First(first), for_application()));
    Ok(())
}

/// What a registry keeps of the station's registrations for the application: each under the grant
/// it wrote for it, and whether that grant is still open.
#[derive(Default)]
struct Grants(Vec<(Registration, bool)>);

impl Grants {
    /// Registers the station under a new grant, `grant`, written at `registered`, and answers the
    /// registration with it.
    fn register(&mut self, grant: u8, registered: i64) -> Result<Vec<u8>, Box<dyn Error>> {
        let written = registration(grant, registered);
        self.0.push((written.clone(), true));
        Ok(EnvelopeBuilder::new().add(&for_application())?.add_must_understand(&written)?.encode())
    }

    /// Ends the registration a notice that the station no longer shares names: its grant alone, if
    /// it is still open, so that a notice under an old grant leaves a newer one open.
    fn stop_sharing(&mut self, notice: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let received = registry_reads(notice);
        let (_, application, named) = received.components().read::<(StopSharing, ForApplication, Registration)>()?;
        assert_eq!(application, for_application());
        for (registration, open) in &mut self.0 {
            if registration.grant == named.grant {
                *open = false;
            }
        }
        Ok(done())
    }

    fn open(&self) -> Vec<Registration> {
        self.0.iter().filter(|(_, open)| *open).map(|(registration, _)| registration.clone()).collect()
    }
}

/// The registration a station keeps from the registry's answer to its `Register`.
fn kept(answer: &[u8]) -> Result<Registration, Box<dyn Error>> {
    let (application, registration) = decoded(answer, REPLIES_UNDERSTOOD).components().read::<(ForApplication, Registration)>()?;
    assert_eq!(application, for_application());
    Ok(registration)
}

/// The notice that the station no longer shares with the application under `registration`.
fn notice(registration: &Registration) -> Result<Vec<u8>, Box<dyn Error>> {
    Ok(EnvelopeBuilder::new().add_must_understand(&StopSharing {})?.add(&for_application())?.add_must_understand(registration)?.encode())
}

/// A person removes the application and adds it again; the station's first notice, repeated after
/// its answer was lost, arrives after the new registration, and leaves that registration open,
/// since it names the grant it was about.
#[test]
fn a_notice_that_arrives_late_ends_its_own_registration_and_leaves_a_newer_one_open() -> Outcome {
    let mut grants = Grants::default();
    let first = kept(&grants.register(40, NOW - 10 * MINUTE)?)?;
    let removal = notice(&first)?;
    assert_done(&grants.stop_sharing(&removal)?)?;
    let second = kept(&grants.register(41, NOW - 5 * MINUTE)?)?;
    assert_ne!(first.grant, second.grant, "a registration after a removal is under a new grant");

    assert_done(&grants.stop_sharing(&removal)?)?;
    assert_eq!(grants.open(), std::slice::from_ref(&second));

    let mut hello = EnvelopeBuilder::new();
    for open in grants.open() {
        hello.add_record(registered(&open)?)?;
    }
    let hello = hello.encode();
    let received = decoded(&hello, REPLIES_UNDERSTOOD);
    let listed: Vec<Registration> =
        received.records().map(|record| record.components().read::<Registration>()).collect::<Result<_, _>>()?;
    assert_eq!(listed, [second]);
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

    let received = registry_reads(&request);
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
    registry_reads(&request).components().read::<GetCatalog>()?;

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

    let received = decoded(&answer, REPLIES_UNDERSTOOD);
    let read: Vec<(Option<Media>, Option<PrinterFingerprint>)> =
        received.records().map(|record| record.components().read()).collect::<Result<_, _>>()?;
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
    let received = registry_reads(&request);
    let (_, read_assignment, read_first, application) =
        received.components().read::<(Register, ScopeAssignment, FirstScope, Option<ForApplication>)>()?;
    assert_eq!((read_assignment, read_first, application), (assignment, first, None));
    Ok(())
}
