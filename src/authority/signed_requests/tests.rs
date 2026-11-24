//! A request signed whole, as every station checks one: one that keeps every rule is honoured,
//! and each rule broken alone refuses it.

use ankurah::{
    envelope::{Envelope, Limits, Signature},
    property::Ref,
    EntityId,
};
use ankurah_commons::{
    property::Timestamp,
    requests::{Request, RequestId},
};
use ed25519_dalek::{Signer, SigningKey};

use super::*;
use crate::{jobs::operations::DoWeigh, jobs::ForDevice, public_key::PublicKeys};

/// The moment the station checks the request at, by its clock, in milliseconds since 1970.
const NOW: i64 = 1_794_600_000_000;
const MINUTE: i64 = 60_000;

fn signing_key(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn key(seed: u8) -> PublicKey {
    PublicKey::from(signing_key(seed).verifying_key())
}

/// The application that signs the requests, which the station shares its devices with.
fn application() -> SigningKey {
    signing_key(1)
}

/// The station that checks the requests.
fn station() -> PublicKey {
    key(2)
}

/// A weigh signed by `signer` for `stations`, expiring at `expires`, as the station receives it.
fn signed_weigh(signer: &SigningKey, stations: &[PublicKey], expires: i64) -> Envelope {
    let mut weigh = Envelope::new((
        Request { request_id: RequestId::new() },
        DoWeigh {},
        ForDevice { device: Ref::new(EntityId::from_bytes([12; 32])) },
        AtStations { stations: PublicKeys::new(stations.to_vec()).unwrap() },
        Expiry { expires: Timestamp::from_unix_millis(expires) },
    ))
    .unwrap();
    let identity = PublicKey::from(signer.verifying_key()).identity();
    let signature = signer.sign(&weigh.signed_bytes(&identity));
    weigh.add_signature(&identity, &Signature::Ed25519(signature.to_bytes())).unwrap();
    Envelope::decode(weigh.bytes().clone(), &Limits::default()).unwrap()
}

/// `request`, read as a station reads a weigh and checked at `NOW` as signed by the application.
fn checked(request: &Envelope) -> Result<(), SignedRequestRefused> {
    let (_, _, _, stations, expiry) = request.get::<(Request, DoWeigh, ForDevice, AtStations, Expiry)>().unwrap();
    let (application, station) = (PublicKey::from(application().verifying_key()), station());
    check_signed_request(
        request,
        &stations,
        &expiry,
        &SignedRequestCheck { application: &application, station: &station, now: Timestamp::from_unix_millis(NOW) },
    )
}

#[test]
fn a_request_its_application_signed_for_this_station_alone_and_unexpired_is_honoured() {
    assert_eq!(checked(&signed_weigh(&application(), &[station()], NOW + 5 * MINUTE)), Ok(()));
}

// A key the station does not take for the application carries no authority, whatever it signed.
#[test]
fn a_request_another_key_signed_is_refused() {
    assert_eq!(checked(&signed_weigh(&signing_key(9), &[station()], NOW + 5 * MINUTE)), Err(SignedRequestRefused::NotSigned));
}

// So that a copy of a signed request is honoured at no other station.
#[test]
fn a_request_signed_for_another_station_or_for_more_than_one_is_refused() {
    assert_eq!(checked(&signed_weigh(&application(), &[key(5)], NOW + 5 * MINUTE)), Err(SignedRequestRefused::AnotherStation));
    let both = signed_weigh(&application(), &[station(), key(5)], NOW + 5 * MINUTE);
    assert_eq!(checked(&both), Err(SignedRequestRefused::AnotherStation));
}

// The longest a signed request runs is ten minutes from now, with the drift of clocks beyond; it
// has passed once that drift is past its expiry.
#[test]
fn a_request_is_honoured_until_its_expiry_and_for_no_longer_than_a_station_allows() {
    let skew = Expiry::CLOCK_SKEW_MS;
    assert_eq!(checked(&signed_weigh(&application(), &[station()], NOW - skew)), Ok(()));
    assert_eq!(checked(&signed_weigh(&application(), &[station()], NOW - skew - 1)), Err(SignedRequestRefused::Expired));
    let longest = NOW + SIGNED_REQUEST_MAX_LIFE_MS + skew;
    assert_eq!(checked(&signed_weigh(&application(), &[station()], longest)), Ok(()));
    assert_eq!(checked(&signed_weigh(&application(), &[station()], longest + 1)), Err(SignedRequestRefused::RunsTooLong));
}
