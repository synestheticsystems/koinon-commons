//! What a party asks a station on the direct path, over its own connection, with no registry
//! between them: under a permission slip the application signed for the party's key, signed whole
//! by a process that holds the application's key, or signed by a computer the station's person
//! paired with it, with the claim of the invitation that paired it.

use koinon_commons::{
    authority::InvitationSecret,
    codec::{ReadError, RecordBuilder},
    devices::{WeightStatus, WeightUnit, WeightValue},
    jobs::AttemptState,
};

use super::{
    checks::{offered, opened_signed_request, reaches, under_slip, Approval, Slip},
    *,
};

/// The answer to a listing: a record for each device that `reach` says the request reaches, with
/// the station's own record first.
fn listing(reach: impl Fn(EntityId, MintedId) -> bool) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut answer = EnvelopeBuilder::new();
    let mut own = RecordBuilder::new(Some(station_record()));
    own.add(&station_itself())?;
    answer.add_record(own)?;
    for (device, kind) in offered() {
        if !reach(device, kind) {
            continue;
        }
        let mut record = RecordBuilder::new(Some(device));
        if kind == Print::ID {
            let (device, printer, media) = printer();
            record.add(&device)?.add(&printer)?.add(&media)?;
        } else {
            record.add(&scale())?.add(&Scale {})?;
        }
        answer.add_record(record)?;
    }
    Ok(answer.encode())
}

/// A device as a listing names it: its record's entity, whether it prints, whether its stock is
/// known, and whether it weighs.
type Listed = (Option<EntityId>, bool, bool, bool);

/// The devices a listing names, as the party reads it, after the station's own record.
fn listed(answer: &[u8]) -> Result<Vec<Listed>, Box<dyn Error>> {
    let reply = decoded(answer, REPLIES_UNDERSTOOD);
    let mut records = reply.records();
    let own = records.next().expect("the station's own record comes first");
    assert_eq!((own.entity(), own.components().read::<Station>()?), (Some(station_record()), station_itself()));
    let devices = records
        .map(|record| {
            let (device, printer, media, scale) = record.components().read::<(Device, Option<Printer>, Option<Media>, Option<Scale>)>()?;
            assert_eq!(device.station.id(), station_record(), "the device names the station's record");
            Ok((record.entity(), printer.is_some(), media.is_some(), scale.is_some()))
        })
        .collect::<Result<_, ReadError>>()?;
    Ok(devices)
}

/// The listing a station answers a request under a slip with, at the station's devices.
fn listed_under(slip: &Slip, approval: &Approval) -> Result<Vec<Listed>, Box<dyn Error>> {
    listed(&listing(|device, kind| reaches(approval, Some(slip), device, kind))?)
}

/// A party with a slip for one printer learns that printer's id, and its station's, with no
/// registry to ask.
#[test]
fn a_listing_under_a_slip_is_answered_with_a_record_for_each_device_it_covers() -> Outcome {
    let presented = weeks_slip(&[printer_record()])?;
    let request = EnvelopeBuilder::new().add_must_understand(&ListDevices {})?.add_must_understand(&presented)?.encode();

    let received = station_reads(&request);
    received.components().read::<ListDevices>()?;
    let (slip, approval) = under_slip(&received, None)?;
    assert_eq!(listed_under(&slip, &approval)?, [(Some(printer_record()), true, true, false)]);
    Ok(())
}

/// A slip that allows prints alone lets its party list the station's printers, and no scale.
#[test]
fn a_listing_under_a_slip_for_prints_alone_names_the_printers_alone() -> Outcome {
    let presented = presented_slip(NOW - MINUTE, NOW + 60 * MINUTE, &[Print::ID], &[])?;
    let request = EnvelopeBuilder::new().add_must_understand(&ListDevices {})?.add_must_understand(&presented)?.encode();

    let (slip, approval) = under_slip(&station_reads(&request), None)?;
    assert_eq!(listed_under(&slip, &approval)?, [(Some(printer_record()), true, true, false)]);
    Ok(())
}

/// A process that holds the key of an application that came through a registry lists every device
/// the station offers, signed for that station alone.
#[test]
fn a_signed_listing_is_answered_with_every_device_the_station_offers() -> Outcome {
    let request = signed_request(|body| {
        body.add_must_understand(&ListDevices {})?;
        Ok(())
    })?;
    let read = checks::carried_signed_request(&carrying(&request)?)?;
    let (body, approval) = opened_signed_request(&read)?;
    body.components().read::<ListDevices>()?;

    let answer = listing(|device, kind| reaches(&approval, None, device, kind))?;
    assert_eq!(listed(&answer)?, [(Some(printer_record()), true, true, false), (Some(scale_record()), false, false, true)]);
    Ok(())
}

/// A computer the station's person paired with it, and offered the printer alone, lists the
/// printer and no other device under its own signature.
#[test]
fn a_paired_computers_signed_listing_names_only_the_devices_its_person_offered_it() -> Outcome {
    let request = signed_request_by(&paired_computer(), &[station()], NOW + 5 * MINUTE, |body| {
        body.add_must_understand(&ListDevices {})?;
        Ok(())
    })?;
    let read = checks::carried_signed_request(&carrying(&request)?)?;
    let (body, approval) = opened_signed_request(&read)?;
    body.components().read::<ListDevices>()?;

    let answer = listing(|device, kind| reaches(&approval, None, device, kind))?;
    assert_eq!(listed(&answer)?, [(Some(printer_record()), true, true, false)]);
    Ok(())
}

/// The same computer's weigh on the scale it was not offered is refused, though its signature
/// verifies: a signature reaches no further than its application's approval.
#[test]
fn a_paired_computers_signed_weigh_on_a_scale_it_was_not_offered_is_refused() -> Outcome {
    let device = ForDevice { device: scale_record().into() };
    let request = signed_request_by(&paired_computer(), &[station()], NOW + 5 * MINUTE, |body| {
        body.add_must_understand(&Weigh {})?.add(&device)?;
        Ok(())
    })?;
    let read = checks::carried_signed_request(&carrying(&request)?)?;
    let (body, approval) = opened_signed_request(&read)?;
    let (_, asked) = body.components().read::<(Weigh, ForDevice)>()?;
    assert!(!reaches(&approval, None, asked.device.id(), Weigh::ID));

    let refusal = Refusal { code: Refusal::NOT_AUTHORIZED.into(), message: "this station honours no authority the request carries".into() };
    let answer = EnvelopeBuilder::new().add(&refusal)?.encode();
    assert_eq!(refusal_code(&answer)?, Refusal::NOT_AUTHORIZED);
    Ok(())
}

/// A party's print under its slip: the request beside the slip, its document after the envelope,
/// and the job's attempt back.
#[test]
fn a_print_under_a_slip_carries_the_slip_beside_the_request() -> Outcome {
    let print = label_print();
    let presented = weeks_slip(&[printer_record()])?;
    let mut request = EnvelopeBuilder::new();
    print.add_to(&mut request)?;
    request.add_must_understand(&presented)?;
    let request = request.encode();

    let received = station_reads(&request);
    let (slip, approval) = under_slip(&received, None)?;
    let read = received.components().read::<ReadPrint>()?;
    assert!(reaches(&approval, Some(&slip), read.2.device.id(), Print::ID));
    print.read_back(read);

    let accepted = JobAttempt { state: AttemptState::Accepted, accepted: at(NOW + 100), finished: None };
    let answer = EnvelopeBuilder::new().add(&accepted)?.encode();
    assert_eq!(decoded(&answer, REPLIES_UNDERSTOOD).components().read::<JobAttempt>()?, accepted);
    Ok(())
}

/// A party's weigh under a slip for the whole station, which needs no list of devices.
#[test]
fn a_weigh_under_a_slip_for_the_whole_station_is_answered_with_the_reading() -> Outcome {
    let presented = weeks_slip(&[])?;
    let device = ForDevice { device: scale_record().into() };
    let request = EnvelopeBuilder::new().add_must_understand(&Weigh {})?.add(&device)?.add_must_understand(&presented)?.encode();

    let received = station_reads(&request);
    let (slip, approval) = under_slip(&received, None)?;
    assert!(slip.devices.is_none(), "the slip covers every device at the station");
    assert_eq!(received.components().read::<(Weigh, ForDevice)>()?.1, device);
    assert!(reaches(&approval, Some(&slip), scale_record(), Weigh::ID));

    let reading = ScaleReading { status: WeightStatus::BelowZero, value: WeightValue::new(-0.5)?, unit: WeightUnit::Pounds };
    let answer = EnvelopeBuilder::new().add(&reading)?.encode();
    assert_eq!(decoded(&answer, REPLIES_UNDERSTOOD).components().read::<ScaleReading>()?, reading);
    Ok(())
}

/// Whoever learns that an application's key has leaked hands a station its retirement directly,
/// and the station acknowledges it whether or not it shares with the application.
#[test]
fn a_retirement_handed_over_directly_is_acknowledged_whoever_hands_it_over() -> Outcome {
    let retirement: SignedRetirement = signed(EnvelopeBuilder::new().add_must_understand(&Retirement { retired: at(NOW) })?);
    let request = EnvelopeBuilder::new().add_must_understand(&Retire {})?.add_must_understand(&retirement)?.encode();
    let (_, read) = station_reads(&request).components().read::<(Retire, SignedRetirement)>()?;
    assert_eq!(read, retirement);
    assert_eq!(read.open(&Limits::default(), RETIREMENT_UNDERSTANDS)?.components().read::<Retirement>()?, Retirement { retired: at(NOW) });
    assert_done(&done())
}

/// A computer claims the invitation the station's person made for it, and learns the station's
/// name and the devices the invitation offers, here the printer alone.
#[test]
fn a_claim_of_an_invitation_is_answered_with_the_station_and_the_devices_it_offers() -> Outcome {
    let claim = InvitationClaim { secret: InvitationSecret::from_bytes([0x5c; 16]), name: Name::new("Front desk Mac")? };
    let request = EnvelopeBuilder::new().add_must_understand(&ClaimInvitation {})?.add(&claim)?.encode();

    let received = station_reads(&request);
    assert_eq!(checks::requested(&received, &[])?, ClaimInvitation::ID);
    let (_, read) = received.components().read::<(ClaimInvitation, InvitationClaim)>()?;
    assert_eq!(read, claim);

    // The claim pairs the computer, which may then use what the invitation offered.
    let approval = checks::approval(&key_of(&paired_computer())).expect("the station now shares with the computer");
    let answer = listing(|device, kind| reaches(&approval, None, device, kind))?;
    assert_eq!(listed(&answer)?, [(Some(printer_record()), true, true, false)]);
    Ok(())
}
