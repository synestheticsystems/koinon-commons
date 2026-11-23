//! What a station's checks of a request's authority honour and refuse, at their edges: the one
//! station a signed request names, how far off its expiry lies, and a slip's moments by its
//! issuer's clock and by the station's, the same checks whichever path brought the request; and
//! the registry an application's endorsement vouches for.

use super::{
    checks::{carried_signed_request, checked_slip, opened_signed_request, taken_hosting, Refused},
    *,
};

/// A weigh on the scale, signed by `key` for `stations` until `expires`, as a station opens it.
fn signed_weigh(key: &SigningKey, stations: &[PublicKey], expires: i64) -> Result<Result<(), Refused>, Box<dyn Error>> {
    let request = signed_request_by(key, stations, expires, |body| {
        body.add_must_understand(&Weigh {})?.add(&ForDevice { device: scale_record().into() })?;
        Ok(())
    })?;
    let read = carried_signed_request(&carrying(&request)?)?;
    Ok(opened_signed_request(&read).map(|_| ()))
}

/// A weigh under a slip issued at `issued` and expiring at `expires`, as a station checks it, where
/// the application's slips were last voided at `voided`, if ever.
fn weigh_under_slip(issued: i64, expires: i64, voided: Option<i64>) -> Result<Result<(), Refused>, Box<dyn Error>> {
    let presented = presented_slip(issued, expires, &[Weigh::ID], &[])?;
    let request = EnvelopeBuilder::new()
        .add_must_understand(&Weigh {})?
        .add(&ForDevice { device: scale_record().into() })?
        .add_must_understand(&presented)?
        .encode();
    Ok(checked_slip(&decoded(&request, STATION_UNDERSTANDS), voided.map(at)).map(|_| ()))
}

fn not_authorized(outcome: Result<(), Refused>) -> bool {
    outcome.is_err_and(|refused| refused.code == Refusal::NOT_AUTHORIZED)
}

#[test]
fn a_signed_request_is_honoured_only_at_the_one_station_it_names() -> Outcome {
    assert_eq!(signed_weigh(&application(), &[station()], NOW + 5 * MINUTE)?, Ok(()));
    assert!(not_authorized(signed_weigh(&application(), &[station(), key(4)], NOW + 5 * MINUTE)?));
    assert!(not_authorized(signed_weigh(&application(), &[key(4)], NOW + 5 * MINUTE)?));
    Ok(())
}

/// The longest a signed request runs is ten minutes from now, with a minute beyond for the drift
/// of clocks; it has passed once that minute is past its expiry.
#[test]
fn a_signed_request_is_honoured_until_its_expiry_and_for_no_longer_than_a_station_allows() -> Outcome {
    let longest = NOW + SignedRequest::MAX_LIFE_MS + Expiry::CLOCK_SKEW_MS;
    assert_eq!(signed_weigh(&application(), &[station()], longest)?, Ok(()));
    assert!(not_authorized(signed_weigh(&application(), &[station()], longest + 1)?));
    assert!(not_authorized(signed_weigh(&application(), &[station()], i64::MAX)?));
    assert_eq!(signed_weigh(&application(), &[station()], NOW - Expiry::CLOCK_SKEW_MS)?, Ok(()));
    assert!(not_authorized(signed_weigh(&application(), &[station()], NOW - Expiry::CLOCK_SKEW_MS - 1)?));
    Ok(())
}

/// A key the station shares nothing with, or a signature its signer did not make, carries no
/// authority, whatever it signed.
#[test]
fn a_request_signed_by_an_application_the_station_does_not_share_with_or_by_no_one_is_refused() -> Outcome {
    assert!(not_authorized(signed_weigh(&SigningKey::from_bytes(&[9; 32]), &[station()], NOW + 5 * MINUTE)?));

    let request = signed_request(|body| {
        body.add_must_understand(&Weigh {})?.add(&ForDevice { device: scale_record().into() })?;
        Ok(())
    })?;
    let mut altered = request.content.as_bytes().to_vec();
    *altered.last_mut().unwrap() ^= 1;
    let forged = SignedRequest::new(SealedEnvelope::new(altered), request.signer, request.signature);
    assert!(not_authorized(opened_signed_request(&forged).map(|_| ())));
    Ok(())
}

/// A slip issued at the moment the registry wrote the station's registration is honoured; one
/// issued a millisecond before, under an earlier registration, is not, however soon after a
/// removal the person said yes again.
#[test]
fn a_slip_is_honoured_from_the_moment_of_the_registration_it_rests_on() -> Outcome {
    assert_eq!(weigh_under_slip(REGISTERED, NOW + 60 * MINUTE, None)?, Ok(()));
    assert!(not_authorized(weigh_under_slip(REGISTERED - 1, NOW + 60 * MINUTE, None)?));
    Ok(())
}

/// A registry's voiding ends every slip of the application issued before it, and none issued at or
/// after it.
#[test]
fn a_slip_issued_before_its_applications_slips_were_voided_is_refused() -> Outcome {
    let voided = NOW - 10 * MINUTE;
    assert_eq!(weigh_under_slip(voided, NOW + 60 * MINUTE, Some(voided))?, Ok(()));
    assert!(not_authorized(weigh_under_slip(voided - 1, NOW + 60 * MINUTE, Some(voided))?));
    Ok(())
}

/// A slip's issue is compared with the station's clock allowing a minute's drift, and its expiry
/// with its issue exactly, since one clock wrote both.
#[test]
fn a_slips_moments_are_honoured_up_to_their_edges_and_no_further() -> Outcome {
    let days = PermissionSlip::MAX_LIFE_MS;
    assert_eq!(weigh_under_slip(NOW + Expiry::CLOCK_SKEW_MS, NOW + 60 * MINUTE, None)?, Ok(()));
    assert!(not_authorized(weigh_under_slip(NOW + Expiry::CLOCK_SKEW_MS + 1, NOW + 60 * MINUTE, None)?));
    assert_eq!(weigh_under_slip(NOW - MINUTE, NOW - MINUTE + days, None)?, Ok(()));
    assert!(not_authorized(weigh_under_slip(NOW - MINUTE, NOW - MINUTE + days + 1, None)?));
    assert!(not_authorized(weigh_under_slip(i64::MIN, i64::MAX, None)?));
    assert!(not_authorized(weigh_under_slip(REGISTERED, NOW - Expiry::CLOCK_SKEW_MS - 1, None)?));
    Ok(())
}

/// A slip issued now that lapses at the moment of its issue is honoured for the drift of clocks;
/// one that lapses a millisecond before its issue is refused, though that drift would cover it,
/// since no issuer means a slip that ends before it begins.
#[test]
fn a_slip_that_expires_before_it_was_issued_is_refused() -> Outcome {
    assert_eq!(weigh_under_slip(NOW, NOW, None)?, Ok(()));
    assert!(not_authorized(weigh_under_slip(NOW, NOW - 1, None)?));
    Ok(())
}

/// A registry that issues a slip and voids its application's slips in one millisecond gives the
/// voiding the next millisecond, so the slip is void with the rest; and once it restarts with its
/// clock reading earlier, it carries on from the voiding's moment, so a slip it issues then is
/// honoured.
#[test]
fn a_slip_issued_in_the_millisecond_of_a_voiding_and_before_it_is_void() -> Outcome {
    let now = NOW - 10 * MINUTE;
    let mut clock = RegistryClock::default();
    let issued = clock.moment(now);
    let voided = clock.moment(now);
    assert!(not_authorized(weigh_under_slip(issued, NOW + 60 * MINUTE, Some(voided))?));

    let reissued = RegistryClock::resumed(voided).moment(now - 5);
    assert_eq!(weigh_under_slip(reissued, NOW + 60 * MINUTE, Some(voided))?, Ok(()));
    Ok(())
}

/// The connection proves the party's key, and a slip is of use to its party alone.
#[test]
fn a_slip_presented_by_another_party_is_refused() -> Outcome {
    let mut body = EnvelopeBuilder::new();
    body.add_must_understand(&PermissionSlip { issued: at(NOW - MINUTE), slip_id: entity(30) })?
        .add_must_understand(&Party { key: key(4), name: Name::new("Someone else")? })?
        .add_must_understand(&AtStations { stations: PublicKeys::new(vec![station()])? })?
        .add_must_understand(&AllowsRequests { requests: ComponentIds::new(vec![Weigh::ID])? })?
        .add_must_understand(&Expiry { expires: at(NOW + 60 * MINUTE) })?;
    let presented: PresentedSlip = signed(&body);
    let request = EnvelopeBuilder::new()
        .add_must_understand(&Weigh {})?
        .add(&ForDevice { device: scale_record().into() })?
        .add_must_understand(&presented)?
        .encode();
    assert!(not_authorized(checked_slip(&decoded(&request, STATION_UNDERSTANDS), None).map(|_| ())));
    Ok(())
}

/// An endorsement vouches for the one registry it names: listed in another registry's document, a
/// well-formed endorsement, signed by the application, is not taken, so that no registry lists an
/// application that did not endorse it.
#[test]
fn an_endorsement_is_taken_only_from_the_document_of_the_registry_it_names() -> Outcome {
    let hosting = Hosting { registry: registry(), name: Name::new("Acme")?, address: HostName::new("acme.example.com")? };
    let endorsement: SignedHosting = signed(EnvelopeBuilder::new().add_must_understand(&hosting)?);
    assert_eq!(taken_hosting(&endorsement, &registry())?, hosting);
    assert!(not_authorized(taken_hosting(&endorsement, &key(7)).map(|_| ())));
    Ok(())
}
