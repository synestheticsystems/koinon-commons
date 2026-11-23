//! What a registry asks a station down the connection the station holds to it: a caller's print
//! or weigh through its gateway, signed whole with the application's key, and the news of an
//! application's retirement or of its voided slips.

use koinon_commons::{
    devices::{WeightStatus, WeightUnit, WeightValue},
    jobs::AttemptState,
};

use super::{
    checks::{carried_signed_request, opened_signed_request},
    *,
};

/// What a station answers a print with: a refusal, or the job's attempt with how it failed when it
/// did.
type Answered = (Option<Refusal>, JobAttempt, Option<JobFailure>);

/// What a station answers a print with, read as the registry reads it to pass it on to the caller.
fn read_attempt(answer: &[u8]) -> Result<Answered, Box<dyn Error>> {
    Ok(decoded(answer, REPLIES_UNDERSTOOD).components().read()?)
}

/// A caller's print through a registry's gateway, signed whole, read by the station from inside
/// the signature, its document taken from after the envelope, and the job's attempt back once the
/// station has handed the job to the printer.
#[test]
fn a_gateway_print_is_signed_whole_and_answered_with_its_sent_attempt() -> Outcome {
    let print = label_print();
    let request = signed_request(|body| print.add_to(body))?;

    let read = carried_signed_request(&carrying(&request)?)?;
    assert_eq!(read, request);
    let (body, _) = opened_signed_request(&read)?;
    print.read_back(body.components().read()?);

    let sent = JobAttempt { state: AttemptState::Sent, accepted: at(NOW + 200), finished: Some(at(NOW + 450)) };
    let answer = EnvelopeBuilder::new().add(&sent)?.encode();
    assert_eq!(read_attempt(&answer)?, (None, sent, None));
    Ok(())
}

/// The answer to a print whose attempt failed: the attempt, and how it failed, which a station
/// gives the registry in place of a refusal, since it took the job.
#[test]
fn a_print_whose_attempt_failed_is_answered_with_the_attempt_and_its_failure() -> Outcome {
    let failed = JobAttempt { state: AttemptState::Failed, accepted: at(NOW + 200), finished: Some(at(NOW + 1_200)) };
    let failure = JobFailure { code: JobFailure::DEVICE_FAILED.into(), message: "out of labels".into() };
    let answer = EnvelopeBuilder::new().add(&failed)?.add(&failure)?.encode();
    assert_eq!(read_attempt(&answer)?, (None, failed, Some(failure)));
    Ok(())
}

/// A caller's weigh through a registry's gateway, which a party that holds the application's key
/// may send a station directly in the same form.
#[test]
fn a_gateway_weigh_is_signed_whole_and_answered_with_the_reading() -> Outcome {
    let device = ForDevice { device: scale_record().into() };
    let request = signed_request(|body| {
        body.add_must_understand(&Weigh {})?.add(&device)?;
        Ok(())
    })?;
    let read = carried_signed_request(&carrying(&request)?)?;
    let (body, _) = opened_signed_request(&read)?;
    assert_eq!(body.components().read::<(Weigh, ForDevice)>()?.1, device);

    let reading = ScaleReading { status: WeightStatus::Stable, value: WeightValue::new(1.25)?, unit: WeightUnit::Kilograms };
    let answer = EnvelopeBuilder::new().add(&reading)?.encode();
    assert_eq!(decoded(&answer, REPLIES_UNDERSTOOD).components().read::<ScaleReading>()?, reading);
    Ok(())
}

/// An application's retirement, which a registry passes on down its held connection as soon as
/// the key retires, and which proves itself by its own signature.
#[test]
fn a_retirement_is_passed_on_as_its_key_signed_it_and_acknowledged() -> Outcome {
    let retirement: SignedRetirement = signed(EnvelopeBuilder::new().add_must_understand(&Retirement { retired: at(NOW) })?);
    let request = EnvelopeBuilder::new().add_must_understand(&Retire {})?.add_must_understand(&retirement)?.encode();

    let received = decoded(&request, STATION_UNDERSTANDS);
    let (_, read) = received.components().read::<(Retire, SignedRetirement)>()?;
    let retired = read.open(&Limits::default(), RETIREMENT_UNDERSTANDS)?;
    assert_eq!(retired.must_understand(), [Retirement::ID]);
    assert_eq!(retired.components().read::<Retirement>()?, Retirement { retired: at(NOW) });
    assert_eq!(read.signer, key_of(&application()), "the retirement retires its signer");
    assert_done(&done())
}

/// A registry's voiding of an application's slips, which a station takes from that registry
/// alone.
#[test]
fn a_voiding_of_slips_names_the_application_and_the_moment() -> Outcome {
    let application = ForApplication { key: key_of(&application()) };
    let voided = SlipsVoided { before: at(NOW) };
    let request = EnvelopeBuilder::new().add_must_understand(&VoidSlips {})?.add_must_understand(&voided)?.add(&application)?.encode();

    let received = decoded(&request, STATION_UNDERSTANDS);
    let (_, read_voided, read_application) = received.components().read::<(VoidSlips, SlipsVoided, ForApplication)>()?;
    assert_eq!((read_voided, read_application), (voided, application));
    assert_done(&done())
}

/// What a server of Pylon 0.1.0 asks a station, a print and a weigh with no authority beside
/// them, takes the same components, so that the new protocol loses nothing the old one says.
#[test]
fn pylon_0_1_0s_print_and_weigh_take_the_same_components_without_an_authority() -> Outcome {
    let print = label_print();
    let mut request = EnvelopeBuilder::new();
    print.add_to(&mut request)?;
    let request = request.encode();
    print.read_back(decoded(&request, STATION_UNDERSTANDS).components().read()?);

    let device = ForDevice { device: scale_record().into() };
    let request = EnvelopeBuilder::new().add_must_understand(&Weigh {})?.add(&device)?.encode();
    assert_eq!(decoded(&request, STATION_UNDERSTANDS).components().read::<(Weigh, ForDevice)>()?.1, device);
    Ok(())
}
