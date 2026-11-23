//! What a receiver refuses, and what it ignores: a component it must understand and does not,
//! answered with every such component named; a component or a property its request requires and
//! does not carry, named; a value its type's rule refuses; components that contradict each other;
//! a document that is not the one its print names; and a component it need not understand, which
//! it reads past.

use ankurah::property::PropertyError;
use koinon_commons::{
    authority::NotOpened,
    codec::{DecodeError, EncodeError, Properties, PropertyWriter, ReadError, ValueProblem},
};

use super::{
    checks::{carried_signed_request, checked_slip, opened_signed_request, place, requested, taken_document},
    *,
};

/// A restriction a later sender adds to its slips, which no station of this build knows: that the
/// slip is honoured only between two hours of the day. Its ids are no mint's.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "tests.during_hours")]
#[model(id = "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE")]
#[model(no_ffi)]
pub struct DuringHours {
    #[property(id = "AgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgI")]
    pub opens: i64,
    #[property(id = "AwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwM")]
    pub closes: i64,
}

/// A note a later sender adds to its prints, which a station of this build does not know and
/// need not understand. Its ids are no mint's.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "tests.packing_note")]
#[model(id = "BAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQ")]
#[model(no_ffi)]
pub struct PackingNote {
    #[active_type(LWW)]
    #[property(id = "BQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQU")]
    pub note: String,
}

/// A kind of request a later station takes, to cut a roll of labels, which no station of this
/// build knows. Its id is no mint's.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "tests.cut")]
#[model(id = "BgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgY")]
#[model(no_ffi)]
pub struct Cut {}

/// `PrintJob` as a sender writes it that holds the job's facts in looser types of its own, and
/// may leave the document's hash out: under `PrintJob`'s ids, so that a station reads it as one.
#[derive(Model, Debug, Clone)]
#[model(label = "tests.loose_print_job")]
#[model(no_ffi)]
pub struct LoosePrintJob {
    #[active_type(LWW)]
    pub document_format: String,
    pub copies: i64,
    #[active_type(LWW)]
    pub document_hash: Option<String>,
}

impl Minted for LoosePrintJob {
    const ID: MintedId = PrintJob::ID;

    fn write_properties(&self, properties: &mut PropertyWriter) -> Result<(), EncodeError> {
        properties.write(property::<PrintJob>("document_format"), &self.document_format)?;
        properties.write(property::<PrintJob>("copies"), &self.copies)?;
        properties.write(property::<PrintJob>("document_hash"), &self.document_hash)
    }

    fn read_properties(properties: &Properties<'_>) -> Result<Self, ReadError> {
        Ok(LoosePrintJob {
            document_format: properties.read(property::<PrintJob>("document_format"))?,
            copies: properties.read(property::<PrintJob>("copies"))?,
            document_hash: properties.read(property::<PrintJob>("document_hash"))?,
        })
    }
}

/// `FirstScope` as a sender writes it that names the scope with any text: under `FirstScope`'s
/// ids, so that a registry reads it as one.
#[derive(Model, Debug, Clone)]
#[model(label = "tests.loose_first_scope")]
#[model(no_ffi)]
pub struct LooseFirstScope {
    #[active_type(LWW)]
    pub name: String,
}

impl Minted for LooseFirstScope {
    const ID: MintedId = FirstScope::ID;

    fn write_properties(&self, properties: &mut PropertyWriter) -> Result<(), EncodeError> {
        properties.write(property::<FirstScope>("name"), &self.name)
    }

    fn read_properties(properties: &Properties<'_>) -> Result<Self, ReadError> {
        Ok(LooseFirstScope { name: properties.read(property::<FirstScope>("name"))? })
    }
}

/// `Hosting` as a registry writes it that gives the application's address as any text: under
/// `Hosting`'s ids, so that a station reads it as one.
#[derive(Model, Debug, Clone)]
#[model(label = "tests.loose_hosting")]
#[model(no_ffi)]
pub struct LooseHosting {
    #[active_type(LWW)]
    pub registry: PublicKey,
    #[active_type(LWW)]
    pub name: Name,
    #[active_type(LWW)]
    pub address: String,
}

impl Minted for LooseHosting {
    const ID: MintedId = Hosting::ID;

    fn write_properties(&self, properties: &mut PropertyWriter) -> Result<(), EncodeError> {
        properties.write(property::<Hosting>("registry"), &self.registry)?;
        properties.write(property::<Hosting>("name"), &self.name)?;
        properties.write(property::<Hosting>("address"), &self.address)
    }

    fn read_properties(properties: &Properties<'_>) -> Result<Self, ReadError> {
        Ok(LooseHosting {
            registry: properties.read(property::<Hosting>("registry"))?,
            name: properties.read(property::<Hosting>("name"))?,
            address: properties.read(property::<Hosting>("address"))?,
        })
    }
}

/// A slip with `extra` among its components, listed as one a receiver must understand, presented
/// beside a weigh.
fn weigh_under_a_slip_with<C: Minted>(extra: &C) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut body = EnvelopeBuilder::new();
    body.add_must_understand(&PermissionSlip { issued: at(NOW), slip_id: entity(30) })?
        .add_must_understand(&Party { key: party(), name: Name::new("Warehouse label PC")? })?
        .add_must_understand(&AtStations { stations: PublicKeys::new(vec![station()])? })?
        .add_must_understand(&AllowsRequests { requests: ComponentIds::new(vec![Weigh::ID])? })?
        .add_must_understand(&Expiry { expires: at(NOW + 60 * MINUTE) })?
        .add_must_understand(extra)?;
    let presented: PresentedSlip = signed(&body);
    Ok(EnvelopeBuilder::new()
        .add_must_understand(&Weigh {})?
        .add(&ForDevice { device: scale_record().into() })?
        .add_must_understand(&presented)?
        .encode())
}

/// A slip with a restriction the station does not know: it refuses the request before reading
/// anything of the slip, and its refusal names the restriction, so that the sender learns in one
/// answer what this station is too old for.
#[test]
fn a_restriction_the_station_does_not_know_is_refused_and_named() -> Outcome {
    let request = weigh_under_a_slip_with(&DuringHours { opens: 8, closes: 18 })?;
    let received = decoded(&request, STATION_UNDERSTANDS);
    let read: PresentedSlip = received.components().read()?;
    let refused = read.open(&Limits::default(), SLIP_UNDERSTANDS).unwrap_err();
    let components = vec![DuringHours::ID];
    assert_eq!(refused, NotOpened::Unreadable(DecodeError::NotUnderstood { components: components.clone() }));

    let refusal = Refusal { code: Refusal::NOT_UNDERSTOOD.into(), message: "this station does not know a restriction of the slip".into() };
    let not_understood = NotUnderstood { components: ComponentIds::new(components)? };
    let answer = EnvelopeBuilder::new().add(&refusal)?.add(&not_understood)?.encode();
    let reply = decoded(&answer, REPLIES_UNDERSTOOD);
    let (read_refusal, read_not_understood) = reply.components().read::<(Refusal, NotUnderstood)>()?;
    assert_eq!((read_refusal, read_not_understood.components.ids()), (refusal, [DuringHours::ID].as_slice()));
    Ok(())
}

/// A component the station knows on another path, which no reader of slips acts on, refuses a slip
/// that lists it as one a receiver must understand: each kind of statement is read under its own
/// list, so that a slip can carry nothing its reader would drop.
#[test]
fn a_slip_that_lists_a_component_its_reader_does_not_act_on_is_refused_naming_it() -> Outcome {
    let request = weigh_under_a_slip_with(&DocumentFollows { length: DocumentLength::new(6)? })?;
    let received = decoded(&request, STATION_UNDERSTANDS);
    let presented: PresentedSlip = received.components().read()?;
    let refused = presented.open(&Limits::default(), SLIP_UNDERSTANDS).unwrap_err();
    assert_eq!(refused, NotOpened::Unreadable(DecodeError::NotUnderstood { components: vec![DocumentFollows::ID] }));
    assert_eq!(checked_slip(&received, None).err().expect("the slip is refused").code, Refusal::NOT_UNDERSTOOD);
    Ok(())
}

/// A request whose own component the receiver does not know is refused by that component's id,
/// with every other must-understand component it does not know: here a later kind of request,
/// under a later restriction.
#[test]
fn a_request_the_station_does_not_know_is_refused_with_every_component_it_does_not_know_named() -> Outcome {
    let request = EnvelopeBuilder::new()
        .add_must_understand(&Cut {})?
        .add(&ForDevice { device: printer_record().into() })?
        .add_must_understand(&DuringHours { opens: 8, closes: 18 })?
        .encode();
    let refused = ReceivedEnvelope::decode(&request, &Limits::default(), STATION_UNDERSTANDS).unwrap_err();
    let mut unknown = vec![Cut::ID, DuringHours::ID];
    unknown.sort();
    assert_eq!(refused, DecodeError::NotUnderstood { components: unknown });
    Ok(())
}

/// A component a receiver need not understand, which a later sender adds, is read past, and the
/// request reads as it would without it.
#[test]
fn a_component_the_station_need_not_understand_is_read_past() -> Outcome {
    let print = label_print();
    let mut request = EnvelopeBuilder::new();
    print.add_to(&mut request)?;
    request.add(&PackingNote { note: "fragile".into() })?;
    let request = request.encode();

    let received = decoded(&request, STATION_UNDERSTANDS);
    assert!(received.components().ids().any(|id| id == PackingNote::ID));
    print.read_back(received.components().read()?);
    Ok(())
}

/// A print without the device it is for is refused by the missing component's id.
#[test]
fn a_print_without_the_device_it_is_for_is_refused_naming_the_missing_component() -> Outcome {
    let print = label_print();
    let request = EnvelopeBuilder::new()
        .add_must_understand(&Print {})?
        .add(&print.job)?
        .add(&print.print_job)?
        .add_must_understand(&print.follows)?
        .encode();
    let refused =
        decoded(&request, STATION_UNDERSTANDS).components().read::<(Print, JobId, ForDevice, PrintJob, DocumentFollows)>().unwrap_err();
    assert!(matches!(refused, ReadError::MissingComponent { component } if component == ForDevice::ID), "{refused}");
    Ok(())
}

/// A print whose particulars lack the document's hash is refused by the ids of the component and
/// the missing property; and one that asks for no copies, or names its hash in another text, is
/// refused by the rule of the property's type, as a stored record of it would be.
#[test]
fn a_print_job_that_lacks_a_property_or_breaks_its_types_rule_is_refused_naming_it() -> Outcome {
    let read = |print_job: &LoosePrintJob| -> Result<PrintJob, ReadError> {
        let request = EnvelopeBuilder::new().add(print_job).unwrap().encode();
        decoded(&request, STATION_UNDERSTANDS).components().read::<PrintJob>()
    };
    let hash = DocumentHash::from_bytes([0xab; 32]).text();
    let loose = |copies, document_hash| LoosePrintJob { document_format: "application/x-zpl2".into(), copies, document_hash };

    assert_eq!(read(&loose(2, Some(hash.clone())))?.copies, Copies::new(2)?);
    let refused = read(&loose(2, None)).unwrap_err();
    let missing = property::<PrintJob>("document_hash");
    assert!(
        matches!(refused, ReadError::MissingProperty { component, property } if component == PrintJob::ID && property == missing),
        "{refused}"
    );
    for (copies, document_hash, field) in
        [(0, hash.clone(), "copies"), (-5, hash.clone(), "copies"), (2, hash.to_uppercase(), "document_hash")]
    {
        let refused = read(&loose(copies, Some(document_hash))).unwrap_err();
        let expected = property::<PrintJob>(field);
        assert!(
            matches!(&refused, ReadError::MalformedValue { component, property: malformed, problem: ValueProblem::Refused(error) }
                if *component == PrintJob::ID && *malformed == expected && matches!(**error, PropertyError::InvalidValue { .. })),
            "{refused}"
        );
    }
    Ok(())
}

/// A first scope named with nothing but white space gives a registry nothing to list it by, and
/// is refused by the rule of a name, naming the property.
#[test]
fn a_first_scope_with_a_blank_name_is_refused_naming_the_property() -> Outcome {
    let request = EnvelopeBuilder::new().add(&LooseFirstScope { name: " \t".into() })?.encode();
    let refused = decoded(&request, REGISTRY_UNDERSTANDS).components().read::<FirstScope>().unwrap_err();
    let name = property::<FirstScope>("name");
    assert!(
        matches!(&refused, ReadError::MalformedValue { component, property, problem: ValueProblem::Refused(error) }
            if *component == FirstScope::ID && *property == name && matches!(**error, PropertyError::InvalidValue { .. })),
        "{refused}"
    );
    Ok(())
}

/// An endorsement whose address is more than a host name, or a host name written another way,
/// though the application's key signed it, is refused by the rule of a host name, naming the
/// property, so that a station never compares such an address with the one its person typed.
#[test]
fn an_endorsement_of_an_address_that_is_no_host_name_is_refused_naming_the_property() -> Outcome {
    let address = property::<Hosting>("address");
    for written in ["https://acme.example.com", "acme.example.com/login", "Acme.example.com", ""] {
        let hosting = LooseHosting { registry: registry(), name: Name::new("Acme")?, address: written.into() };
        let endorsement: SignedHosting = signed(EnvelopeBuilder::new().add_must_understand(&hosting)?);
        let opened = endorsement.open(&Limits::default(), HOSTING_UNDERSTANDS)?;
        let refused = opened.components().read::<Hosting>().unwrap_err();
        assert!(
            matches!(&refused, ReadError::MalformedValue { component, property, problem: ValueProblem::Refused(error) }
                if *component == Hosting::ID && *property == address && matches!(**error, PropertyError::InvalidValue { .. })),
            "{written:?}: {refused}"
        );
    }
    Ok(())
}

/// A request asks one thing: one that carries two request components is refused, rather than
/// answered as either.
#[test]
fn a_request_that_asks_two_things_is_refused_as_a_bad_request() -> Outcome {
    let device = ForDevice { device: scale_record().into() };
    let request = EnvelopeBuilder::new().add_must_understand(&Weigh {})?.add_must_understand(&ListDevices {})?.add(&device)?.encode();
    let refused = requested(&decoded(&request, STATION_UNDERSTANDS).components()).unwrap_err();
    assert_eq!(refused.code, Refusal::BAD_REQUEST, "{refused}");
    Ok(())
}

/// A registration names one place for the station: one that carries both a placement and a first
/// scope is refused, rather than registered in either.
#[test]
fn a_registration_in_a_listed_scope_and_a_first_scope_at_once_is_refused_as_a_bad_request() -> Outcome {
    let request = EnvelopeBuilder::new()
        .add_must_understand(&Register {})?
        .add(&ScopeAssignment { station: station_record().into() })?
        .add(&Placement { scope: entity(20).into(), sub_scope: None })?
        .add(&FirstScope { name: Name::new("Acme")? })?
        .add_must_understand(&ForApplication { key: key_of(&application()) })?
        .encode();
    let refused = place(&decoded(&request, REGISTRY_UNDERSTANDS).components()).unwrap_err();
    assert_eq!(refused.code, Refusal::BAD_REQUEST, "{refused}");
    Ok(())
}

/// A request has its authority from a signature over it or from a slip beside it, never from both:
/// a signed request that carries a slip inside is refused, rather than honoured under either.
#[test]
fn a_signed_request_that_carries_a_slip_inside_is_refused_as_a_bad_request() -> Outcome {
    let presented = weeks_slip(&[])?;
    let request = signed_request(|body| {
        body.add_must_understand(&Weigh {})?.add(&ForDevice { device: scale_record().into() })?.add_must_understand(&presented)?;
        Ok(())
    })?;
    let read = carried_signed_request(&carrying(&request)?)?;
    let refused = opened_signed_request(&read).err().expect("the request is refused");
    assert_eq!(refused.code, Refusal::BAD_REQUEST, "{refused}");
    Ok(())
}

/// A station takes the document after a print's envelope only whole and only as the print names
/// it: a stream that ends early, or a document altered on the way, is refused before anything
/// reaches the printer.
#[test]
fn a_document_that_ends_early_or_is_not_the_one_its_print_names_is_refused() -> Outcome {
    let print = label_print();
    let short = &print.document[..print.document.len() - 1];
    let mut altered = print.document.to_vec();
    altered[4] ^= 1;
    for stream in [short, &altered] {
        let refused = taken_document(&print.follows, &print.print_job, stream).unwrap_err();
        assert_eq!(refused.code, Refusal::BAD_REQUEST, "{refused}");
    }
    // What follows the document on the stream is the next frame, which the document leaves alone.
    let longer = [print.document, b"next"].concat();
    assert_eq!(taken_document(&print.follows, &print.print_job, &longer)?, print.document);
    Ok(())
}
