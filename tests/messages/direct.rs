//! What a party asks a station on the direct path, over its own connection, with no registry
//! between them: under a permission slip the application signed for the party's key, or signed
//! whole by a process that holds the application's key.

use koinon_commons::{
    authority::DeviceIds,
    codec::{ReadError, RecordBuilder},
    component_ids::ComponentIds,
    devices::{WeightStatus, WeightUnit, WeightValue},
    jobs::AttemptState,
    public_key::PublicKeys,
};

use super::*;

/// A week's slip for the party, at the station, for prints and weighs, narrowed to `devices`
/// when there are any.
fn slip(devices: &[EntityId]) -> Result<(PermissionSlip, PresentedSlip), Box<dyn Error>> {
    let body = PermissionSlip { party: party(), party_name: "Warehouse label PC".into(), issued: at(NOW - MINUTE), slip_id: entity(30) };
    let mut envelope = EnvelopeBuilder::new();
    envelope
        .add_must_understand(&body)?
        .add_must_understand(&AtStations { stations: PublicKeys::new(vec![station()])? })?
        .add_must_understand(&AllowsRequests { requests: ComponentIds::new(vec![Print::ID, Weigh::ID])? })?
        .add_must_understand(&Expiry { expires: at(NOW + 7 * 24 * 60 * MINUTE) })?;
    if !devices.is_empty() {
        envelope.add_must_understand(&AtDevices { devices: DeviceIds::new(devices.iter().map(|&device| device.into()).collect())? })?;
    }
    Ok((body, signed(&envelope)))
}

/// What a station reads of a request's slip before it acts on the request: the carrier, whose
/// signature it checks over the bytes as they arrived, then the slip's body and restrictions.
fn checked_slip(received: &ReceivedEnvelope<'_>, request: MintedId) -> Result<Option<AtDevices>, Box<dyn Error>> {
    let presented: PresentedSlip = received.components().read()?;
    presented.verify()?;
    assert_eq!(presented.signer, PublicKey::from(application().verifying_key()));
    let slip = presented.content.decode(&Limits::default(), STATION_UNDERSTANDS)?;
    let (body, stations, devices, allows, expiry) =
        slip.components().read::<(PermissionSlip, AtStations, Option<AtDevices>, AllowsRequests, Expiry)>()?;
    // The connection proves the party's key.
    assert_eq!(body.party, party());
    assert!(body.issued.unix_millis() <= NOW && expiry.expires.unix_millis() - body.issued.unix_millis() <= PermissionSlip::MAX_LIFE_MS);
    assert!(!expiry.has_passed(at(NOW)));
    assert!(stations.stations.contains(&station()));
    assert!(allows.requests.contains(request) || request == ListDevices::ID);
    Ok(devices)
}

/// A party with a slip for one printer learns that printer's id, and its station's, with no
/// registry to ask.
#[test]
fn a_listing_under_a_slip_is_answered_with_a_record_for_each_device_it_covers() -> Outcome {
    let (_, presented) = slip(&[printer_record()])?;
    let request = EnvelopeBuilder::new().add_must_understand(&ListDevices {})?.add_must_understand(&presented)?.encode();

    let received = decoded(&request, STATION_UNDERSTANDS);
    received.components().read::<ListDevices>()?;
    let covered = checked_slip(&received, ListDevices::ID)?.expect("the slip narrows to devices");
    assert_eq!(covered.devices.devices(), [printer_record().into()]);

    let (device, printer, media) = printer();
    let mut own = RecordBuilder::new(Some(station_record()));
    own.add(&Station { public_key: station().text(), name: "Pack bench 3".into() })?;
    let mut offered = RecordBuilder::new(Some(printer_record()));
    offered.add(&device)?.add(&printer)?.add(&media)?;
    let mut answer = EnvelopeBuilder::new();
    answer.add_record(own)?.add_record(offered)?;
    let answer = answer.encode();

    let reply = decoded(&answer, REPLIES_UNDERSTOOD);
    let records: Vec<_> = reply.records().collect();
    assert_eq!(records.iter().map(|record| record.entity()).collect::<Vec<_>>(), [Some(station_record()), Some(printer_record())]);
    let read_station: Station = records[0].components().read()?;
    assert_eq!(read_station.public_key, station().text());
    let (read_device, read_printer, read_media, read_scale): (Device, Option<Printer>, Option<Media>, Option<Scale>) =
        records[1].components().read()?;
    assert_eq!(read_device.station.id(), records[0].entity().unwrap(), "the device names the station's record");
    assert_eq!(
        [json(&read_device), json(&read_printer.unwrap()), json(&read_media.unwrap())],
        [json(&device), json(&printer), json(&media)]
    );
    assert!(read_scale.is_none());
    Ok(())
}

/// A process that holds the application's key lists every device the station offers, signed for
/// that station alone.
#[test]
fn a_signed_listing_is_answered_with_every_device_the_station_offers() -> Outcome {
    let mut body = EnvelopeBuilder::new();
    body.add_must_understand(&ListDevices {})?
        .add_must_understand(&AtStations { stations: PublicKeys::new(vec![station()])? })?
        .add_must_understand(&Expiry { expires: at(NOW + 5 * MINUTE) })?;
    let request: SignedRequest = signed(&body);
    let bytes = EnvelopeBuilder::new().add_must_understand(&request)?.encode();

    let received = decoded(&bytes, STATION_UNDERSTANDS);
    let read: SignedRequest = received.components().read()?;
    read.verify()?;
    let body = read.content.decode(&Limits::default(), STATION_UNDERSTANDS)?;
    let (_, stations, expiry) = body.components().read::<(ListDevices, AtStations, Expiry)>()?;
    assert!(stations.stations.contains(&station()) && !expiry.has_passed(at(NOW)));

    let (device, printer, _) = printer();
    let mut offered = RecordBuilder::new(Some(printer_record()));
    offered.add(&device)?.add(&printer)?;
    let mut weighs = RecordBuilder::new(Some(scale_record()));
    weighs.add(&scale())?.add(&Scale {})?;
    let mut answer = EnvelopeBuilder::new();
    answer.add_record(offered)?.add_record(weighs)?;
    let answer = answer.encode();

    let reply = decoded(&answer, REPLIES_UNDERSTOOD);
    let kinds: Vec<(Option<EntityId>, bool, bool, bool)> = reply
        .records()
        .map(|record| {
            let (device, printer, media, scale) = record.components().read::<(Device, Option<Printer>, Option<Media>, Option<Scale>)>()?;
            assert_eq!(device.station.id(), station_record());
            Ok((record.entity(), printer.is_some(), media.is_some(), scale.is_some()))
        })
        .collect::<Result<_, ReadError>>()?;
    // A printer no one has said the stock of comes without `Media`.
    assert_eq!(kinds, [(Some(printer_record()), true, false, false), (Some(scale_record()), false, false, true)]);
    Ok(())
}

/// A party's print under its slip: the request beside the slip, its document after the envelope,
/// and the job's attempt back.
#[test]
fn a_print_under_a_slip_carries_the_slip_beside_the_request() -> Outcome {
    let print = label_print();
    let (_, presented) = slip(&[printer_record()])?;
    let mut request = EnvelopeBuilder::new();
    print.add_to(&mut request)?;
    request.add_must_understand(&presented)?;
    let request = request.encode();

    let received = decoded(&request, STATION_UNDERSTANDS);
    let covered = checked_slip(&received, Print::ID)?.expect("the slip narrows to devices");
    let read = received.components().read::<(Print, JobId, ForDevice, Option<JobLabel>, PrintJob, DocumentFollows)>()?;
    assert!(covered.devices.contains(&read.2.device));
    print.read_back(read);

    let accepted = JobAttempt { state: AttemptState::Accepted, accepted: at(NOW + 100), finished: None };
    let answer = EnvelopeBuilder::new().add(&accepted)?.encode();
    assert_eq!(decoded(&answer, REPLIES_UNDERSTOOD).components().read::<JobAttempt>()?, accepted);
    Ok(())
}

/// A party's weigh under a slip for the whole station, which needs no list of devices.
#[test]
fn a_weigh_under_a_slip_for_the_whole_station_is_answered_with_the_reading() -> Outcome {
    let (_, presented) = slip(&[])?;
    let device = ForDevice { device: scale_record().into() };
    let request = EnvelopeBuilder::new().add_must_understand(&Weigh {})?.add(&device)?.add_must_understand(&presented)?.encode();

    let received = decoded(&request, STATION_UNDERSTANDS);
    assert_eq!(checked_slip(&received, Weigh::ID)?, None, "the slip holds for every device at the station");
    assert_eq!(received.components().read::<(Weigh, ForDevice)>()?.1, device);

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
    let (_, read) = decoded(&request, STATION_UNDERSTANDS).components().read::<(Retire, SignedRetirement)>()?;
    assert_eq!(read, retirement);
    read.verify()?;
    assert_done(&done())
}
