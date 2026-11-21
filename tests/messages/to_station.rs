//! What a registry asks a station down the connection the station holds to it: a caller's print
//! or weigh through its gateway, signed whole with the application's key, and the news of an
//! application's retirement or of its voided slips.

use koinon_commons::{
    devices::{WeightStatus, WeightUnit, WeightValue},
    jobs::AttemptState,
    public_key::PublicKeys,
};

use super::*;

/// `components` in a request signed with the application's key for this station alone, for the
/// five minutes a gateway gives a job to reach a station.
fn signed_request(components: impl FnOnce(&mut EnvelopeBuilder) -> Outcome) -> Result<SignedRequest, Box<dyn Error>> {
    let mut body = EnvelopeBuilder::new();
    components(&mut body)?;
    body.add_must_understand(&AtStations { stations: PublicKeys::new(vec![station()])? })?
        .add_must_understand(&Expiry { expires: at(NOW + 5 * MINUTE) })?;
    Ok(signed(&body))
}

/// The envelope a signed request travels in: the carrier alone, listed so that a station too old
/// to check a signature refuses it rather than ignores it.
fn carrying(request: &SignedRequest) -> Result<Vec<u8>, Box<dyn Error>> {
    Ok(EnvelopeBuilder::new().add_must_understand(request)?.encode())
}

/// What a station reads of a signed request before it acts: the carrier, whose signature it
/// checks over the bytes as they arrived, then the restrictions inside.
fn checked(bytes: &[u8]) -> Result<SignedRequest, Box<dyn Error>> {
    let received = decoded(bytes, STATION_UNDERSTANDS);
    assert_eq!(received.must_understand(), [SignedRequest::ID]);
    let request: SignedRequest = received.components().read()?;
    request.verify()?;
    assert_eq!(request.signer, PublicKey::from(application().verifying_key()));
    let body = request.content.decode(&Limits::default(), STATION_UNDERSTANDS)?;
    let (stations, expiry) = body.components().read::<(AtStations, Expiry)>()?;
    assert_eq!(stations.stations.keys(), [station()]);
    assert!(!expiry.has_passed(at(NOW)) && expiry.expires.unix_millis() - NOW <= SignedRequest::MAX_LIFE_MS);
    Ok(request)
}

/// A caller's print through a registry's gateway: the print signed whole, its document after the
/// envelope, and the job's attempt back, first as sent and then, after a repeat, as failed.
#[test]
fn a_gateway_print_is_signed_whole_and_answered_with_the_jobs_attempt() -> Outcome {
    let print = label_print();
    let request = signed_request(|body| print.add_to(body))?;
    let bytes = carrying(&request)?;

    let read = checked(&bytes)?;
    assert_eq!(read, request);
    let body = read.content.decode(&Limits::default(), STATION_UNDERSTANDS)?;
    print.read_back(body.components().read()?);

    let sent = JobAttempt { state: AttemptState::Sent, accepted: at(NOW + 200), finished: Some(at(NOW + 450)) };
    let answer = EnvelopeBuilder::new().add(&sent)?.encode();
    let reply = decoded(&answer, REPLIES_UNDERSTOOD);
    assert_eq!(reply.components().read::<(Option<Refusal>, JobAttempt, Option<JobFailure>)>()?, (None, sent, None));

    let failed = JobAttempt { state: AttemptState::Failed, accepted: at(NOW + 200), finished: Some(at(NOW + 1_200)) };
    let failure = JobFailure { code: "device_failed".into(), message: "out of labels".into() };
    let answer = EnvelopeBuilder::new().add(&failed)?.add(&failure)?.encode();
    let reply = decoded(&answer, REPLIES_UNDERSTOOD);
    assert_eq!(reply.components().read::<(JobAttempt, Option<JobFailure>)>()?, (failed, Some(failure)));
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
    let read = checked(&carrying(&request)?)?;
    let body = read.content.decode(&Limits::default(), STATION_UNDERSTANDS)?;
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
    read.verify()?;
    let retired = read.content.decode(&Limits::default(), STATION_UNDERSTANDS)?;
    assert_eq!(retired.must_understand(), [Retirement::ID]);
    assert_eq!(retired.components().read::<Retirement>()?, Retirement { retired: at(NOW) });
    assert_done(&done())
}

/// A registry's voiding of an application's slips, which a station takes from that registry
/// alone.
#[test]
fn a_voiding_of_slips_names_the_application_and_the_moment() -> Outcome {
    let application = ForApplication { key: PublicKey::from(application().verifying_key()) };
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
