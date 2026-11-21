//! What a receiver refuses, and what it ignores: a component it must understand and does not,
//! answered with every such component named; a component or a property its request requires and
//! does not hold, named; a value its type's rule refuses; and a component it need not understand,
//! which it reads past.

use ankurah::property::PropertyError;
use koinon_commons::{
    codec::{DecodeError, EncodeError, Properties, PropertyWriter, ReadError, ValueProblem},
    component_ids::ComponentIds,
    public_key::PublicKeys,
};

use super::*;

/// A restriction a later sender adds to its slips, which no station of this build knows: that the
/// slip holds only between two hours of the day. Its ids are no mint's.
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

/// A slip with a restriction the station does not know: it refuses the request before reading
/// anything of the slip, and its refusal names the restriction, so that the sender learns in one
/// answer what this station is too old for.
#[test]
fn a_restriction_the_station_does_not_know_is_refused_and_named() -> Outcome {
    let mut body = EnvelopeBuilder::new();
    body.add_must_understand(&PermissionSlip {
        party: party(),
        party_name: "Warehouse label PC".into(),
        issued: at(NOW),
        slip_id: entity(30),
    })?
    .add_must_understand(&AtStations { stations: PublicKeys::new(vec![station()])? })?
    .add_must_understand(&AllowsRequests { requests: ComponentIds::new(vec![Weigh::ID])? })?
    .add_must_understand(&Expiry { expires: at(NOW + 60 * MINUTE) })?
    .add_must_understand(&DuringHours { opens: 8, closes: 18 })?;
    let presented: PresentedSlip = signed(&body);
    let request = EnvelopeBuilder::new()
        .add_must_understand(&Weigh {})?
        .add(&ForDevice { device: scale_record().into() })?
        .add_must_understand(&presented)?
        .encode();

    let received = decoded(&request, STATION_UNDERSTANDS);
    let read: PresentedSlip = received.components().read()?;
    read.verify()?;
    let refused = read.content.decode(&Limits::default(), STATION_UNDERSTANDS).unwrap_err();
    assert_eq!(refused, DecodeError::NotUnderstood { components: vec![DuringHours::ID] });

    let DecodeError::NotUnderstood { components } = refused else { unreachable!() };
    let refusal = Refusal { code: Refusal::NOT_UNDERSTOOD.into(), message: "this station does not know a restriction of the slip".into() };
    let not_understood = NotUnderstood { components: ComponentIds::new(components)? };
    let answer = EnvelopeBuilder::new().add(&refusal)?.add(&not_understood)?.encode();
    let reply = decoded(&answer, REPLIES_UNDERSTOOD);
    let (read_refusal, read_not_understood) = reply.components().read::<(Refusal, NotUnderstood)>()?;
    assert_eq!((read_refusal, read_not_understood.components.ids()), (refusal, [DuringHours::ID].as_slice()));
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
    let hash_property = property::<PrintJob>("document_hash");
    assert!(
        matches!(refused, ReadError::MissingProperty { component, property: missing } if component == PrintJob::ID && missing == hash_property),
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
