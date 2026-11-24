//! What a registry asks a station, down the connection the station holds to it, and how the
//! station answers: a caller's print and weigh, signed whole by the application they come from,
//! with a print's document after the envelope; the news that an application retired its key; and
//! a voiding of an application's slips.

use ankurah_commons::requests::{Request, RequestId, Response};
use koinon_commons::{
    authority::{check_signed_request, ForApplication, Retirement, SlipsVoided},
    devices::{ScaleReading, WeightStatus, WeightUnit, WeightValue},
    jobs::{
        operations::{DoPrint, DoWeigh},
        AttemptState, JobAttempt, JobFailure,
    },
};

use super::*;

/// The gateway's print of the label, signed whole by the application, for this station alone.
fn gateway_print(print: &LabelPrint, asked: &RequestId) -> Result<Envelope, Box<dyn Error>> {
    let (stations, expiry) = this_station_for_five_minutes();
    let request = Envelope::new((
        Request { request_id: asked.clone() },
        DoPrint {},
        print.device.clone(),
        print.job.clone(),
        print.label.clone(),
        stations,
        expiry,
    ))?;
    Ok(signed(request, &application()))
}

/// What the station reads of a gateway's print on `stream`: the envelope, its authority checked
/// before anything more is read, and then the document the print names.
async fn station_takes_print(stream: &[u8]) -> Result<(Request, ForDevice, PrintJob, Option<JobLabel>, Vec<u8>), Box<dyn Error>> {
    let mut stream = stream;
    let envelope = envelope_of(&mut stream).await?;
    let (request, _, device, job, label, stations, expiry) =
        envelope.get::<(Request, DoPrint, ForDevice, PrintJob, Option<JobLabel>, AtStations, Expiry)>()?;
    let (application, station) = (key_of(&application()), station());
    check_signed_request(&envelope, &stations, &expiry, &signed_request_check(&application, &station))?;
    let document = attachment_of(&mut stream, &job.document).await?;
    Ok((request, device, job, label, document))
}

#[tokio::test]
async fn a_gateway_print_is_signed_whole_its_document_follows_and_it_is_answered_with_its_attempt() -> Outcome {
    let print = label_print();
    let asked = RequestId::new();
    let stream = sent(&gateway_print(&print, &asked)?, Some(print.document)).await?;
    let (request, device, job, label, document) = station_takes_print(&stream).await?;
    assert_eq!((device, job, label, document.as_slice()), (print.device, print.job, Some(print.label), print.document));

    let attempt = JobAttempt { state: AttemptState::Sent, accepted: at(NOW), finished: Some(at(NOW + 800)) };
    let answer = Envelope::new((Response { request_id: request.request_id }, attempt.clone()))?;
    let (response, read, failure) = received(&answer).await?.get::<(Response, JobAttempt, Option<JobFailure>)>()?;
    assert_eq!((response.request_id, read, failure), (asked, attempt, None));
    Ok(())
}

// A failed attempt is no refusal: the station took the job, and the failure is its outcome.
#[tokio::test]
async fn a_print_whose_attempt_failed_is_answered_with_the_attempt_and_its_failure() -> Outcome {
    let asked = RequestId::new();
    let attempt = JobAttempt { state: AttemptState::Failed, accepted: at(NOW), finished: Some(at(NOW + 30_000)) };
    let failure = JobFailure { code: JobFailure::DEVICE_TIMEOUT.into(), message: "the printer did not answer".into() };
    let answer = Envelope::new((Response { request_id: asked.clone() }, attempt.clone(), failure.clone()))?;
    let (response, read, failed) = received(&answer).await?.get::<(Response, JobAttempt, Option<JobFailure>)>()?;
    assert_eq!((response.request_id, read, failed), (asked, attempt, Some(failure)));
    Ok(())
}

// The signature covers the document through its hash and its length, so the station prints
// nothing but the bytes the application named.
#[tokio::test]
async fn a_document_that_is_not_the_one_the_signed_print_names_is_refused() -> Outcome {
    let print = label_print();
    let request = gateway_print(&print, &RequestId::new())?;
    let other = b"^XA^FO50,50^ADN,36,20^FDOrder 9999^FS^XZ";
    assert_eq!(other.len(), print.document.len());
    let refused = station_takes_print(&sent(&request, Some(other)).await?).await;
    assert!(refused.is_err_and(|error| matches!(error.downcast_ref::<ReceiveError>(), Some(ReceiveError::WrongHash))));
    let shorter = &print.document[..print.document.len() - 1];
    let refused = station_takes_print(&sent(&request, Some(shorter)).await?).await;
    assert!(refused.is_err_and(|error| matches!(error.downcast_ref::<ReceiveError>(), Some(ReceiveError::LengthMismatch { .. }))));
    let refused = station_takes_print(&sent(&request, None).await?).await;
    assert!(refused.is_err_and(|error| matches!(error.downcast_ref::<ReceiveError>(), Some(ReceiveError::NoAttachment))));
    Ok(())
}

#[tokio::test]
async fn a_gateway_weigh_is_signed_whole_and_answered_with_the_reading() -> Outcome {
    let asked = RequestId::new();
    let (stations, expiry) = this_station_for_five_minutes();
    let weigh = ForDevice { device: Ref::new(scale_record()) };
    let request =
        signed(Envelope::new((Request { request_id: asked.clone() }, DoWeigh {}, weigh.clone(), stations, expiry))?, &application());
    let taken = received(&request).await?;
    let (request, _, device, stations, expiry) = taken.get::<(Request, DoWeigh, ForDevice, AtStations, Expiry)>()?;
    let (application, station) = (key_of(&application()), station());
    check_signed_request(&taken, &stations, &expiry, &signed_request_check(&application, &station))?;
    assert_eq!(device, weigh);

    let reading = ScaleReading { status: WeightStatus::Stable, value: WeightValue::new(1.25)?, unit: WeightUnit::Pounds };
    let answer = Envelope::new((Response { request_id: request.request_id }, reading.clone()))?;
    let (response, read) = received(&answer).await?.get::<(Response, ScaleReading)>()?;
    assert_eq!((response.request_id, read), (asked, reading));
    Ok(())
}

// A retirement proves itself by the signature of the key it retires, so whoever holds it passes it
// on as its exact bytes, by any path, and the station answers whether or not it shares its devices
// with the application.
#[tokio::test]
async fn a_retirement_is_signed_by_the_key_it_retires_passed_on_as_its_bytes_and_answered_with_the_response_alone() -> Outcome {
    let asked = RequestId::new();
    let retirement = Retirement { retired: at(NOW - MINUTE) };
    let signed_retirement = signed(Envelope::new((Request { request_id: asked.clone() }, retirement.clone()))?, &application());

    // The registry passes it on as it came to it.
    let passed_on = received(&signed_retirement).await?;
    assert_eq!(passed_on.bytes(), signed_retirement.bytes());
    let taken = received(&passed_on).await?;
    let (request, read) = taken.get::<(Request, Retirement)>()?;
    assert_eq!(read, retirement);
    assert!(taken.is_signed_by(&key_of(&application()).identity()), "the retirement retires the key that signed it");
    assert!(!taken.is_signed_by(&key(7).identity()));

    let answer = received(&Envelope::new(Response { request_id: request.request_id })?).await?;
    assert_eq!(answer.get::<Response>()?.request_id, asked);
    Ok(())
}

#[tokio::test]
async fn a_voiding_of_slips_names_the_application_and_the_moment_and_is_answered_with_the_response_alone() -> Outcome {
    let asked = RequestId::new();
    let voided = SlipsVoided { before: at(NOW - MINUTE) };
    let application = ForApplication { key: key_of(&application()) };
    let request = Envelope::new((Request { request_id: asked.clone() }, voided.clone(), application.clone()))?;
    let (request, read, of) = received(&request).await?.get::<(Request, SlipsVoided, ForApplication)>()?;
    assert_eq!((read, of), (voided, application));

    let answer = received(&Envelope::new(Response { request_id: request.request_id })?).await?;
    assert_eq!(answer.get::<Response>()?.request_id, asked);
    Ok(())
}

// A server of Pylon 0.1.0 is trusted by its connection, so its requests carry no authority.
#[tokio::test]
async fn pylon_0_1_0s_print_and_weigh_take_the_same_components_without_an_authority() -> Outcome {
    let print = label_print();
    let request = Envelope::new((Request { request_id: RequestId::new() }, DoPrint {}, print.device.clone(), print.job.clone()))?;
    let stream = sent(&request, Some(print.document)).await?;
    let mut stream = stream.as_slice();
    let (_, _, device, job, label) = envelope_of(&mut stream).await?.get::<(Request, DoPrint, ForDevice, PrintJob, Option<JobLabel>)>()?;
    assert_eq!((device, &job, label), (print.device, &print.job, None));
    assert_eq!(attachment_of(&mut stream, &job.document).await?, print.document);

    let weigh = Envelope::new((Request { request_id: RequestId::new() }, DoWeigh {}, ForDevice { device: Ref::new(scale_record()) }))?;
    let (_, _, device) = received(&weigh).await?.get::<(Request, DoWeigh, ForDevice)>()?;
    assert_eq!(device.device, Ref::new(scale_record()));
    Ok(())
}
