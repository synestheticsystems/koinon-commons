//! What a party asks a station on the direct path, over its own connection, with no registry
//! between them: under a permission slip the application signed for the party's key, presented as
//! the request's token; and a computer's claim of an invitation to pair with the station.

use ankurah::envelope::{EntityRows, EnvelopeComponent};
use ankurah_commons::requests::{AuthToken, Request, RequestId, Response};
use koinon_commons::{
    authority::{check_slip, open_slip, slip_covers, slip_token, InvitationClaim, InvitationSecret},
    devices::{operations::DoListDevices, DeviceListing, Scale, ScaleReading, WeightStatus, WeightUnit, WeightValue},
    jobs::{
        operations::{DoPrint, DoWeigh},
        AttemptState, JobAttempt,
    },
};

use super::*;

/// The slip a request's token presents, opened and checked as the station checks it on the party's
/// own connection, giving the devices it narrows to and the operations it allows.
fn slip_of(token: AuthToken) -> Result<(Option<koinon_commons::authority::AtDevices>, AllowsRequests), Box<dyn Error>> {
    let slip = open_slip(token, &Limits::default())?;
    let (application, party, station) = (key_of(&application()), party(), station());
    let (_, _, _, devices, allows, _) = check_slip(&slip, &slip_check(&application, &party, &station))?;
    Ok((devices, allows))
}

#[tokio::test]
async fn a_print_under_a_slip_carries_the_slip_as_its_token_and_its_document_after_the_envelope() -> Outcome {
    let print = label_print();
    let asked = RequestId::new();
    let request = Envelope::new((
        Request { request_id: asked.clone() },
        DoPrint {},
        print.device.clone(),
        print.job.clone(),
        print.label.clone(),
        slip_token(&weeks_slip(&[DoPrint::ID, DoWeigh::ID])),
    ))?;
    let stream = sent(&request, Some(print.document)).await?;

    // The station checks the slip on the envelope, before it reads any of the document.
    let mut stream = stream.as_slice();
    let (request, _, device, job, label, token) =
        envelope_of(&mut stream).await?.get::<(Request, DoPrint, ForDevice, PrintJob, Option<JobLabel>, AuthToken)>()?;
    let (devices, allows) = slip_of(token)?;
    assert!(slip_covers(devices.as_ref(), &allows, DoPrint::ID, &device.device));
    assert_eq!((device, &job, label), (print.device, &print.job, Some(print.label)));
    assert_eq!(attachment_of(&mut stream, &job.document).await?, print.document);

    let attempt = JobAttempt { state: AttemptState::Sent, accepted: at(NOW), finished: Some(at(NOW + 800)) };
    let answer = Envelope::new((Response { request_id: request.request_id }, attempt.clone()))?;
    let (response, read) = received(&answer).await?.get::<(Response, JobAttempt)>()?;
    assert_eq!((response.request_id, read), (asked, attempt));
    Ok(())
}

#[tokio::test]
async fn a_weigh_under_a_slip_is_answered_with_the_reading() -> Outcome {
    let asked = RequestId::new();
    let weigh = ForDevice { device: Ref::new(scale_record()) };
    let request =
        Envelope::new((Request { request_id: asked.clone() }, DoWeigh {}, weigh.clone(), slip_token(&weeks_slip(&[DoWeigh::ID]))))?;
    let (request, _, device, token) = received(&request).await?.get::<(Request, DoWeigh, ForDevice, AuthToken)>()?;
    let (devices, allows) = slip_of(token)?;
    assert!(slip_covers(devices.as_ref(), &allows, DoWeigh::ID, &device.device));
    assert_eq!(device, weigh);

    let reading = ScaleReading { status: WeightStatus::InMotion, value: WeightValue::new(0.4)?, unit: WeightUnit::Kilograms };
    let (response, read) = received(&Envelope::new((Response { request_id: request.request_id }, reading.clone()))?)
        .await?
        .get::<(Response, ScaleReading)>()?;
    assert_eq!((response.request_id, read), (asked, reading));
    Ok(())
}

// A slip that allows prints alone lets its party list the station's printers, and nothing else.
#[tokio::test]
async fn a_listing_under_a_slip_for_prints_names_the_printers_alone_each_under_its_id() -> Outcome {
    let asked = RequestId::new();
    let request = Envelope::new((Request { request_id: asked.clone() }, DoListDevices {}, slip_token(&weeks_slip(&[DoPrint::ID]))))?;
    let (request, _, token) = received(&request).await?.get::<(Request, DoListDevices, AuthToken)>()?;
    let (devices, allows) = slip_of(token)?;

    // The station lists each of its devices the slip covers for the operation of its kind.
    let (printer_device, printer, media) = printer();
    let mut offered = EntityRows::builder();
    offered.push(Some(printer_record()), (printer_device.clone(), printer.clone(), media.clone()))?;
    offered.push(Some(scale_record()), (scale(), Scale {}))?;
    let mut covered = EntityRows::builder();
    for row in rows_of(&offered.build())? {
        let id = row.entity_id().ok_or("a device's row names its device")?;
        let covers = operation_of(&row).is_some_and(|operation| slip_covers(devices.as_ref(), &allows, operation, &Ref::new(id)));
        if covers {
            covered.push(Some(id), row.get::<(Device, Printer, Option<Media>)>()?)?;
        }
    }
    let answer = Envelope::new((Response { request_id: request.request_id }, DeviceListing { entries: covered.build() }))?;

    let (response, listing) = received(&answer).await?.get::<(Response, DeviceListing)>()?;
    assert_eq!(response.request_id, asked);
    let rows = rows_of(&listing.entries)?;
    let listed = rows
        .iter()
        .map(|row| Ok((row.entity_id(), row.get::<(Device, Printer, Option<Media>)>()?)))
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
    assert_eq!(listed, [(Some(printer_record()), (printer_device, printer, Some(media)))]);
    Ok(())
}

// The claim's secret is its whole authority, and the station answers with its own record and the
// devices the invitation offers, so that the computer knows what it may use and where.
#[tokio::test]
async fn a_claim_of_an_invitation_is_answered_with_the_station_and_the_devices_it_offers() -> Outcome {
    let claim = InvitationClaim { secret: InvitationSecret::from_bytes(*b"0123456789abcdef"), name: Name::new("Front desk Mac")? };
    let asked = RequestId::new();
    let (request, taken) =
        received(&Envelope::new((Request { request_id: asked.clone() }, claim.clone()))?).await?.get::<(Request, InvitationClaim)>()?;
    assert_eq!(taken, claim);

    let (printer_device, printer, media) = printer();
    let mut entries = EntityRows::builder();
    entries.push(Some(station_record()), station_itself())?;
    entries.push(Some(printer_record()), (printer_device.clone(), printer.clone(), media.clone()))?;
    let answer = Envelope::new((Response { request_id: request.request_id }, DeviceListing { entries: entries.build() }))?;

    let (response, listing) = received(&answer).await?.get::<(Response, DeviceListing)>()?;
    assert_eq!(response.request_id, asked);
    let rows = rows_of(&listing.entries)?;
    let station_row = rows.iter().find(|row| row.has::<Station>()).ok_or("the answer to a claim holds the station's own row")?;
    assert_eq!((station_row.entity_id(), station_row.get::<Station>()?), (Some(station_record()), station_itself()));
    let devices = rows
        .iter()
        .filter(|row| row.has::<Device>())
        .map(|row| Ok((row.entity_id(), row.get::<(Device, Printer, Option<Media>)>()?)))
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
    assert_eq!(devices, [(Some(printer_record()), (printer_device, printer, Some(media)))]);
    Ok(())
}
