//! A permission slip, as every station checks one: a slip that keeps every rule is honoured, and
//! each rule broken alone refuses it, under the code its refusal names.

use ankurah::{
    envelope::{Envelope, EnvelopeComponent, Limits, ReadError, Signature},
    property::Ref,
    EntityId, Model,
};
use ankurah_commons::{property::Timestamp, requests::AuthToken};
use ed25519_dalek::{Signer, SigningKey};

use super::*;
use crate::{
    component_ids::ComponentIds,
    devices::DeviceIds,
    jobs::operations::{DoPrint, DoWeigh},
    name::Name,
    public_key::{PublicKey, PublicKeys},
    replies::Refusal,
};

/// The moment the station checks the slip at, by its clock, in milliseconds since 1970.
const NOW: i64 = 1_794_600_000_000;
const MINUTE: i64 = 60_000;
/// When the application's registry wrote the station's registration for the application, by the
/// registry's clock: the earliest a slip of the application the station honours was issued.
const REGISTERED: i64 = NOW - 60 * MINUTE;

/// A restriction a later version adds to slips, which this station does not know.
#[derive(Model, EnvelopeComponent, Debug, Clone, PartialEq)]
#[model(id = "BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwc")]
#[model(no_ffi)]
#[envelope(must_understand)]
pub struct AtHours {
    #[active_type(LWW)]
    #[envelope(number = 0)]
    pub hours: String,
}

fn at(millis: i64) -> Timestamp {
    Timestamp::from_unix_millis(millis)
}

fn signing_key(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn key(seed: u8) -> PublicKey {
    PublicKey::from(signing_key(seed).verifying_key())
}

/// The application that signs the slips, which the station shares its devices with.
fn application() -> SigningKey {
    signing_key(1)
}

/// The station that checks the slips.
fn station() -> PublicKey {
    key(2)
}

/// The party the slips are for.
fn party() -> PublicKey {
    key(3)
}

fn printer() -> Ref<Device> {
    Ref::new(EntityId::from_bytes([11; 32]))
}

fn scale() -> Ref<Device> {
    Ref::new(EntityId::from_bytes([12; 32]))
}

/// `envelope` signed by `signer`, as an application signs a slip.
fn signed(mut envelope: Envelope, signer: &SigningKey) -> Envelope {
    let identity = PublicKey::from(signer.verifying_key()).identity();
    let signature = signer.sign(&envelope.signed_bytes(&identity));
    envelope.add_signature(&identity, &Signature::Ed25519(signature.to_bytes())).unwrap();
    envelope
}

fn body(issued: i64) -> PermissionSlip {
    PermissionSlip { issued: at(issued), slip_id: EntityId::from_bytes([30; 32]) }
}

fn for_party(key: PublicKey) -> Party {
    Party { key, name: Name::new("Warehouse label PC").unwrap() }
}

fn at_stations(stations: &[PublicKey]) -> AtStations {
    AtStations { stations: PublicKeys::new(stations.to_vec()).unwrap() }
}

fn allowing(operations: &[EntityId]) -> AllowsRequests {
    AllowsRequests { requests: ComponentIds::new(operations.to_vec()).unwrap() }
}

fn expiring(expires: i64) -> Expiry {
    Expiry { expires: at(expires) }
}

/// A week's slip for the party at this station, issued a minute ago, for prints and weighs,
/// signed by the application.
fn weeks_slip() -> Envelope {
    let slip = Envelope::new((
        body(NOW - MINUTE),
        for_party(party()),
        at_stations(&[station()]),
        allowing(&[DoPrint::ID, DoWeigh::ID]),
        expiring(NOW + 7 * 24 * 60 * MINUTE),
    ))
    .unwrap();
    signed(slip, &application())
}

/// A slip for the party at this station, for prints, issued at `issued` and expiring at `expires`,
/// signed by the application.
fn slip_of_moments(issued: i64, expires: i64) -> Envelope {
    let slip =
        Envelope::new((body(issued), for_party(party()), at_stations(&[station()]), allowing(&[DoPrint::ID]), expiring(expires))).unwrap();
    signed(slip, &application())
}

/// What the station knows when it checks a slip at `NOW` by its clock, on a connection that proves
/// `connection`, honouring the application's slips from `REGISTERED`.
fn check_of<'a>(application: &'a PublicKey, connection: &'a PublicKey, station: &'a PublicKey) -> SlipCheck<'a> {
    SlipCheck { application, party: connection, station, honoured_from: at(REGISTERED), now: at(NOW) }
}

/// Whether the station honours `slip`, presented on a connection that proves `connection`.
fn checked_from(slip: &Envelope, connection: &PublicKey) -> Result<(), SlipRefused> {
    let (application, station) = (PublicKey::from(application().verifying_key()), station());
    check_slip(slip, &check_of(&application, connection, &station)).map(|_| ())
}

fn checked(slip: &Envelope) -> Result<(), SlipRefused> {
    checked_from(slip, &party())
}

/// The refusal, and its code, that checking `slip` gives.
fn refusal(slip: &Envelope) -> (String, &'static str) {
    let refused = checked(slip).unwrap_err();
    (refused.to_string(), refused.code())
}

#[test]
fn a_slip_that_keeps_every_rule_is_honoured_and_reads_back_as_its_application_wrote_it() {
    let slip = weeks_slip();
    let token = slip_token(&slip);
    assert_eq!((token.format, token.credential.as_slice()), (PermissionSlip::ID, slip.bytes().as_ref()));
    let opened = open_slip(token, &Limits::default()).unwrap();
    assert_eq!(opened.bytes(), slip.bytes());
    let (application, connection, at_station) = (PublicKey::from(application().verifying_key()), party(), station());
    let (read, slip_party, stations, devices, allows, expiry) =
        check_slip(&opened, &check_of(&application, &connection, &at_station)).unwrap();
    assert_eq!(read, body(NOW - MINUTE));
    assert_eq!(slip_party, for_party(party()));
    assert_eq!(stations, at_stations(&[station()]));
    assert_eq!(devices, None);
    assert_eq!(allows, allowing(&[DoPrint::ID, DoWeigh::ID]));
    assert_eq!(expiry, expiring(NOW + 7 * 24 * 60 * MINUTE));
}

// A slip signed by a key the station does not take for the application, or by no one, carries no
// authority, however well it reads.
#[test]
fn a_slip_its_application_did_not_sign_is_refused() {
    let unsigned = Envelope::decode(weeks_slip().body().clone(), &Limits::default());
    assert!(unsigned.is_err(), "a body alone is no envelope");
    let parts = (body(NOW - MINUTE), for_party(party()), at_stations(&[station()]), allowing(&[DoPrint::ID]), expiring(NOW + MINUTE));
    let unsigned = Envelope::new(parts.clone()).unwrap();
    assert!(matches!(checked(&unsigned), Err(SlipRefused::NotSigned)));
    let by_another = signed(Envelope::new(parts).unwrap(), &signing_key(9));
    assert!(matches!(checked(&by_another), Err(SlipRefused::NotSigned)));
    assert_eq!(SlipRefused::NotSigned.code(), Refusal::NOT_AUTHORIZED);
}

// The connection proves the party's key, so that a slip is of use to its party alone.
#[test]
fn a_slip_presented_on_another_partys_connection_is_refused() {
    assert!(matches!(checked_from(&weeks_slip(), &key(4)), Err(SlipRefused::AnotherParty)));
}

#[test]
fn a_slip_that_does_not_name_this_station_is_refused() {
    let slip =
        Envelope::new((body(NOW - MINUTE), for_party(party()), at_stations(&[key(5)]), allowing(&[DoPrint::ID]), expiring(NOW + MINUTE)))
            .unwrap();
    assert!(matches!(checked(&signed(slip, &application())), Err(SlipRefused::AnotherStation)));
}

// A slip's issue is compared with the station's clock allowing the drift of clocks, and its
// expiry with its issue exactly, since one clock wrote both.
#[test]
fn a_slip_is_honoured_up_to_the_edges_of_its_moments_and_no_further() {
    let skew = Expiry::CLOCK_SKEW_MS;
    assert!(checked(&slip_of_moments(NOW + skew, NOW + skew)).is_ok());
    assert!(matches!(checked(&slip_of_moments(NOW + skew + 1, NOW + skew + 1)), Err(SlipRefused::IssuedLater)));
    assert!(checked(&slip_of_moments(NOW, NOW)).is_ok());
    assert!(matches!(checked(&slip_of_moments(NOW, NOW - 1)), Err(SlipRefused::ExpiresBeforeIssue)));
    assert!(checked(&slip_of_moments(NOW - MINUTE, NOW - MINUTE + PermissionSlip::MAX_LIFE_MS)).is_ok());
    assert!(matches!(
        checked(&slip_of_moments(NOW - MINUTE, NOW - MINUTE + PermissionSlip::MAX_LIFE_MS + 1)),
        Err(SlipRefused::RunsTooLong)
    ));
    assert!(checked(&slip_of_moments(REGISTERED, NOW + MINUTE)).is_ok());
    assert!(matches!(checked(&slip_of_moments(REGISTERED - 1, NOW + MINUTE)), Err(SlipRefused::IssuedBeforeHonoured)));
    assert!(checked(&slip_of_moments(REGISTERED, NOW - skew)).is_ok());
    assert!(matches!(checked(&slip_of_moments(REGISTERED, NOW - skew - 1)), Err(SlipRefused::Expired)));
}

// So that a station never honours a slip beyond what its application allowed, a restriction the
// station does not know refuses the slip, named, rather than being dropped.
#[test]
fn a_slip_that_holds_a_restriction_the_station_does_not_know_is_refused_naming_it() {
    let slip = Envelope::new((
        body(NOW - MINUTE),
        for_party(party()),
        at_stations(&[station()]),
        allowing(&[DoPrint::ID]),
        expiring(NOW + MINUTE),
        AtHours { hours: "09:00-17:00".into() },
    ))
    .unwrap();
    let refused = checked(&signed(slip, &application())).unwrap_err();
    assert!(
        matches!(&refused, SlipRefused::Unreadable(ReadError::NotUnderstood { components }) if *components == [AtHours::ID]),
        "{refused}"
    );
    assert_eq!(refused.code(), Refusal::NOT_UNDERSTOOD);
}

#[test]
fn a_slip_that_lacks_a_restriction_every_slip_has_is_refused_as_a_bad_request() {
    let slip = Envelope::new((body(NOW - MINUTE), for_party(party()), at_stations(&[station()]), expiring(NOW + MINUTE))).unwrap();
    let (words, code) = refusal(&signed(slip, &application()));
    assert!(words.contains(&AllowsRequests::ID.to_string()), "{words}");
    assert_eq!(code, Refusal::BAD_REQUEST);
}

// A receiver that does not know a token's format names it, and never takes the request for one
// that carries no token.
#[test]
fn a_token_in_another_format_is_refused_naming_the_format() {
    let other = EntityId::from_bytes([8; 32]);
    let refused = open_slip(AuthToken { format: other, credential: weeks_slip().bytes().to_vec() }, &Limits::default()).unwrap_err();
    assert!(matches!(refused, SlipRefused::OtherFormat(format) if format == other));
    assert!(refused.to_string().contains(&other.to_string()), "{refused}");
    assert_eq!(refused.code(), Refusal::NOT_AUTHORIZED);
}

#[test]
fn a_token_whose_credential_is_no_envelope_is_refused_as_a_bad_request() {
    let refused = open_slip(AuthToken { format: PermissionSlip::ID, credential: b"a slip".to_vec() }, &Limits::default()).unwrap_err();
    assert!(matches!(refused, SlipRefused::Undecodable(_)), "{refused}");
    assert_eq!(refused.code(), Refusal::BAD_REQUEST);
}

#[test]
fn a_slip_covers_the_devices_it_names_for_the_operations_it_allows() {
    let prints = allowing(&[DoPrint::ID]);
    assert!(slip_covers(None, &prints, DoPrint::ID, &printer()), "a slip that names no device covers every device");
    assert!(!slip_covers(None, &prints, DoWeigh::ID, &scale()), "nor an operation it does not allow");
    let narrowed = AtDevices { devices: DeviceIds::new(vec![printer()]).unwrap() };
    assert!(slip_covers(Some(&narrowed), &prints, DoPrint::ID, &printer()));
    assert!(!slip_covers(Some(&narrowed), &allowing(&[DoPrint::ID, DoWeigh::ID]), DoWeigh::ID, &scale()), "a device it does not name");
}
