//! What a receiver understands of an envelope, and what it refuses for not understanding it: a
//! component it must understand and does not, answered with every such component named, as is one
//! it must understand where it reads none, or beside a request that does not act on it; and a
//! component it need not understand, which it reads past.

use koinon_commons::{
    authority::NotOpened,
    codec::{DecodeError, RecordBuilder},
};

use super::{
    checks::{carried_signed_request, opened_signed_request, under_slip, Path},
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

/// A slip for weighs at the station, with what `extra` adds to its envelope, presented beside a
/// weigh on the scale.
fn weigh_under_a_slip_with(extra: impl FnOnce(&mut EnvelopeBuilder) -> Outcome) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut body = EnvelopeBuilder::new();
    body.add_must_understand(&PermissionSlip { issued: at(NOW), slip_id: entity(30) })?
        .add_must_understand(&Party { key: party(), name: Name::new("Warehouse label PC")? })?
        .add_must_understand(&AtStations { stations: PublicKeys::new(vec![station()])? })?
        .add_must_understand(&AllowsRequests { requests: ComponentIds::new(vec![Weigh::ID])? })?
        .add_must_understand(&Expiry { expires: at(NOW + 60 * MINUTE) })?;
    extra(&mut body)?;
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
    let request = weigh_under_a_slip_with(|body| {
        body.add_must_understand(&DuringHours { opens: 8, closes: 18 })?;
        Ok(())
    })?;
    let received = station_reads(&request);
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
    let request = weigh_under_a_slip_with(|body| {
        body.add_must_understand(&DocumentFollows { length: DocumentLength::new(6)? })?;
        Ok(())
    })?;
    let received = station_reads(&request);
    let presented: PresentedSlip = received.components().read()?;
    let refused = presented.open(&Limits::default(), SLIP_UNDERSTANDS).unwrap_err();
    assert_eq!(refused, NotOpened::Unreadable(DecodeError::NotUnderstood { components: vec![DocumentFollows::ID] }));
    assert_eq!(under_slip(&received, &party(), None).err().expect("the slip is refused").code, Refusal::NOT_UNDERSTOOD);
    Ok(())
}

/// A slip that holds a restriction in a record, where no reader of slips reads one, and lists it as
/// one a receiver must understand, is refused naming it, rather than honoured as a slip without
/// it: here a slip narrowed to the printer, beside a weigh on the scale, which it would otherwise
/// reach.
#[test]
fn a_slip_that_holds_a_restriction_in_a_record_is_refused_naming_it() -> Outcome {
    let request = weigh_under_a_slip_with(|body| {
        let mut narrowed = RecordBuilder::new(None);
        narrowed.add_must_understand(&AtDevices { devices: DeviceIds::new(vec![printer_record().into()])? })?;
        body.add_record(narrowed)?;
        Ok(())
    })?;
    let received = station_reads(&request);
    let presented: PresentedSlip = received.components().read()?;
    let refused = presented.open(&Limits::default(), SLIP_UNDERSTANDS).unwrap_err();
    assert_eq!(refused, NotOpened::Unreadable(DecodeError::NotUnderstood { components: vec![AtDevices::ID] }));
    assert_eq!(under_slip(&received, &party(), None).err().expect("the slip is refused").code, Refusal::NOT_UNDERSTOOD);
    Ok(())
}

/// A component the station acts on for one kind of request, listed as one a receiver must
/// understand beside a request that does not act on it, refuses that request as not understood,
/// signed whole or beside a slip: here a `DocumentFollows`, which says a document follows on the
/// stream, in a weigh or a listing, whose station reads no document and would take the
/// document's bytes for its next request.
#[test]
fn a_component_listed_beside_a_request_that_does_not_act_on_it_is_refused_as_not_understood() -> Outcome {
    let follows = DocumentFollows { length: DocumentLength::new(6)? };
    let scale = ForDevice { device: scale_record().into() };
    let weigh = signed_request(|body| {
        body.add_must_understand(&Weigh {})?.add(&scale)?.add_must_understand(&follows)?;
        Ok(())
    })?;
    let listing = signed_request(|body| {
        body.add_must_understand(&ListDevices {})?.add_must_understand(&follows)?;
        Ok(())
    })?;
    for request in [weigh, listing] {
        let read = carried_signed_request(&carrying(&request)?)?;
        let refused = opened_signed_request(&read, &Path::Direct).err().expect("the request is refused");
        assert_eq!(refused.code, Refusal::NOT_UNDERSTOOD, "{refused}");
    }

    let beside_a_slip = EnvelopeBuilder::new()
        .add_must_understand(&Weigh {})?
        .add(&scale)?
        .add_must_understand(&follows)?
        .add_must_understand(&weeks_slip(&[])?)?
        .encode();
    let refused = under_slip(&station_reads(&beside_a_slip), &party(), None).err().expect("the request is refused");
    assert_eq!(refused.code, Refusal::NOT_UNDERSTOOD, "{refused}");
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
    let refused = ReceivedEnvelope::decode_own_components(&request, &Limits::default(), STATION_UNDERSTANDS).unwrap_err();
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

    let received = station_reads(&request);
    assert!(received.components().ids().any(|id| id == PackingNote::ID));
    print.read_back(received.components().read()?);
    Ok(())
}
