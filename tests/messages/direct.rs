//! What a party asks a station on the direct path, over its own connection, with no registry
//! between them: under a permission slip the application signed for the party's key, signed whole
//! by a process that holds the application's key, or signed by a computer the station's person
//! paired with it, with the claim of the invitation that paired it.

use koinon_commons::{
    authority::InvitationSecret,
    codec::RecordBuilder,
    devices::{WeightStatus, WeightUnit, WeightValue},
    jobs::AttemptState,
};

use super::{
    checks::{offered, opened_signed_request, reaches, under_slip, Approval, Path, Slip},
    *,
};

/// An answer that names each device `reach` says the request reaches, each in a record under the
/// device's entity id, after the station's own record when the answer carries `station`: how a
/// station answers a listing, which may carry its own record, and a claim of an invitation, which
/// does.
fn devices_answer(station: Option<&Station>, reach: impl Fn(EntityId, MintedId) -> bool) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut answer = EnvelopeBuilder::new();
    if let Some(station) = station {
        let mut own = RecordBuilder::new(Some(station_record()));
        own.add(station)?;
        answer.add_record(own)?;
    }
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

/// What a station answers a listing with, under `approval`, and under `slip` when it came under
/// one: a refusal as `not_authorized` when the slip narrows to devices none of which the station
/// offers, as another station's, and otherwise its own record and a record for each device the
/// request reaches.
fn listing(approval: &Approval, slip: Option<&Slip>) -> Result<Vec<u8>, Box<dyn Error>> {
    let narrowed_to_none_offered = slip
        .and_then(|slip| slip.devices.as_ref())
        .is_some_and(|narrowed| !offered().iter().any(|&(device, _)| narrowed.devices.contains(&device.into())));
    if narrowed_to_none_offered {
        let refusal = Refusal { code: Refusal::NOT_AUTHORIZED.into(), message: "the slip covers no device this station offers".into() };
        return Ok(EnvelopeBuilder::new().add(&refusal)?.encode());
    }
    devices_answer(Some(&station_itself()), |device, kind| reaches(approval, slip, device, kind))
}

/// A device as an answer names it: its record's entity, whether it prints, whether its stock is
/// known, and whether it weighs.
type Listed = (Option<EntityId>, bool, bool, bool);

/// What an answer that names devices says, as the party reads it: the station's own record, when
/// the answer carries it, and each device it names, in the station's order.
#[derive(Debug, PartialEq)]
struct Named {
    station: Option<Station>,
    devices: Vec<Listed>,
}

/// What an answer to a listing, or to a claim of an invitation, names, read as the party reads it:
/// the record that carries `Station` is the station's own, wherever it stands, and every other
/// names a device, which names the station's record. A refusal reads as the refusal.
fn named(answer: &[u8]) -> Result<Result<Named, Refusal>, Box<dyn Error>> {
    reply(answer, |received| {
        let mut named = Named { station: None, devices: Vec::new() };
        for record in received.records() {
            if let Some(station) = record.components().read::<Option<Station>>()? {
                if record.entity() != Some(station_record()) || named.station.replace(station).is_some() {
                    return Err("the station's own record comes once, under its id".into());
                }
                continue;
            }
            let (device, printer, media, scale) = record.components().read::<(Device, Option<Printer>, Option<Media>, Option<Scale>)>()?;
            if device.station.id() != station_record() {
                return Err("a device names another station's record".into());
            }
            named.devices.push((record.entity(), printer.is_some(), media.is_some(), scale.is_some()));
        }
        Ok(named)
    })
}

/// The devices the answer to a listing names, read as the party reads it, which may carry the
/// station's own record or not.
fn listed(answer: &[u8]) -> Result<Vec<Listed>, Box<dyn Error>> {
    Ok(named(answer)?.map_err(|refusal| format!("the listing was refused as {}", refusal.code))?.devices)
}

/// What the answer to a claim of an invitation says, read as the claiming computer reads it: the
/// station's own record, whose name the computer shows its person, and the devices the invitation
/// offers. An answer without the station's record is no answer to a claim.
fn claimed(answer: &[u8]) -> Result<(Station, Vec<Listed>), Box<dyn Error>> {
    let named = named(answer)?.map_err(|refusal| format!("the claim was refused as {}", refusal.code))?;
    Ok((named.station.ok_or("the answer to a claim carries the station's own record")?, named.devices))
}

/// A party with a slip for one printer learns that printer's id, and its station's, with no
/// registry to ask.
#[test]
fn a_listing_under_a_slip_is_answered_with_a_record_for_each_device_it_covers() -> Outcome {
    let presented = weeks_slip(&[printer_record()])?;
    let request = EnvelopeBuilder::new().add_must_understand(&ListDevices {})?.add_must_understand(&presented)?.encode();

    let received = station_reads(&request);
    received.components().read::<ListDevices>()?;
    let (slip, approval) = under_slip(&received, &party(), None)?;
    assert_eq!(listed(&listing(&approval, Some(&slip))?)?, [(Some(printer_record()), true, true, false)]);
    Ok(())
}

/// A slip that allows prints alone lets its party list the station's printers, and no scale.
#[test]
fn a_listing_under_a_slip_for_prints_alone_names_the_printers_alone() -> Outcome {
    let presented = presented_slip(NOW - MINUTE, NOW + 60 * MINUTE, &[Print::ID], &[])?;
    let request = EnvelopeBuilder::new().add_must_understand(&ListDevices {})?.add_must_understand(&presented)?.encode();

    let (slip, approval) = under_slip(&station_reads(&request), &party(), None)?;
    assert_eq!(listed(&listing(&approval, Some(&slip))?)?, [(Some(printer_record()), true, true, false)]);
    Ok(())
}

/// A slip that narrows to devices none of which this station offers, as another station's, is
/// refused as `not_authorized`, rather than answered with no device.
#[test]
fn a_listing_under_a_slip_for_devices_the_station_does_not_offer_is_refused() -> Outcome {
    let presented = weeks_slip(&[entity(99)])?;
    let request = EnvelopeBuilder::new().add_must_understand(&ListDevices {})?.add_must_understand(&presented)?.encode();

    let (slip, approval) = under_slip(&station_reads(&request), &party(), None)?;
    let answer = listing(&approval, Some(&slip))?;
    assert_eq!(named(&answer)?.map_err(|refusal| refusal.code), Err(Refusal::NOT_AUTHORIZED.to_owned()));
    Ok(())
}

/// A station may leave its own record out of a listing, and the party reads the devices it names
/// all the same.
#[test]
fn a_listing_without_the_stations_own_record_names_the_devices_all_the_same() -> Outcome {
    let approval = checks::approval(&key_of(&application())).ok_or("the station shares with the application")?;
    let answer = devices_answer(None, |device, kind| reaches(&approval, None, device, kind))?;
    let devices = vec![(Some(printer_record()), true, true, false), (Some(scale_record()), false, false, true)];
    assert_eq!(named(&answer)?, Ok(Named { station: None, devices }));
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
    let (body, approval) = opened_signed_request(&read, &Path::Direct)?;
    body.components().read::<ListDevices>()?;

    let answer = listing(&approval, None)?;
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
    let (body, approval) = opened_signed_request(&read, &Path::Direct)?;
    body.components().read::<ListDevices>()?;

    let answer = listing(&approval, None)?;
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
    let (body, approval) = opened_signed_request(&read, &Path::Direct)?;
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
    let (slip, approval) = under_slip(&received, &party(), None)?;
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
    let (slip, approval) = under_slip(&received, &party(), None)?;
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
    let answer = devices_answer(Some(&station_itself()), |device, kind| reaches(&approval, None, device, kind))?;
    assert_eq!(claimed(&answer)?, (station_itself(), vec![(Some(printer_record()), true, true, false)]));
    Ok(())
}

/// The computer shows its person the name of the station it paired with from the answer to its
/// claim, so an answer that carries no `Station` record is no answer to a claim, though a listing
/// may leave the record out.
#[test]
fn an_answer_to_a_claim_without_the_stations_own_record_does_not_read_as_one() -> Outcome {
    let approval = checks::approval(&key_of(&paired_computer())).ok_or("the station shares with the computer")?;
    let answer = devices_answer(None, |device, kind| reaches(&approval, None, device, kind))?;
    assert!(claimed(&answer).is_err());
    assert_eq!(listed(&answer)?, [(Some(printer_record()), true, true, false)]);
    Ok(())
}
