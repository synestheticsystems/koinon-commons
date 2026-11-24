//! The checks every station makes of a request signed whole by its application before it acts on
//! it, so that every station honours the same signed requests and refuses the same ones.

use ankurah::envelope::Envelope;
use ankurah_commons::property::Timestamp;

use super::{AtStations, Expiry};
use crate::public_key::PublicKey;

#[cfg(test)]
mod tests;

/// The longest a station honours a request signed whole, in milliseconds: ten minutes past the
/// moment the station checks it, with the drift of clocks beyond. A signed request is honoured
/// whoever delivers it, and a weigh or a listing of devices names no job, so its expiry is all
/// that bounds how long a copy of it can be used; a station refuses one whose expiry lies further
/// off. For a signed print it is also how long the job's id can come again under the signature.
pub const SIGNED_REQUEST_MAX_LIFE_MS: i64 = 10 * 60_000;

/// What a station knows when it checks a request signed whole, and what the request must match.
#[derive(Debug, Clone, Copy)]
pub struct SignedRequestCheck<'a> {
    /// The application, among those the station shares its devices with, whose key the request
    /// names among its signers; for a registry's gateway job, one that came through the registry
    /// whose connection brought it, since a registry's gateway delivers jobs for its own
    /// applications alone.
    pub application: &'a PublicKey,
    /// The station's own key, which the request's `AtStations` must name, and no other.
    pub station: &'a PublicKey,
    /// Now, by the station's clock.
    pub now: Timestamp,
}

/// Checks a request signed whole, `request`, whose `AtStations` and `Expiry` the station read with
/// the rest of what it asks, as every station checks one before it acts on it: the application's
/// key signed the request's body; its `AtStations` names this station and no other, so that a copy
/// of it is honoured nowhere else; and its `Expiry` has not passed, and lies no further off than
/// [`SIGNED_REQUEST_MAX_LIFE_MS`] past now, with [`Expiry::CLOCK_SKEW_MS`] beyond for the drift of
/// clocks. A station refuses each as `not_authorized`.
///
/// A request signed whole takes every fact it acts on from inside the signature, the reference to
/// a print's document among them, so a print's attachment is covered through its hash.
pub fn check_signed_request(
    request: &Envelope,
    stations: &AtStations,
    expiry: &Expiry,
    check: &SignedRequestCheck<'_>,
) -> Result<(), SignedRequestRefused> {
    if !request.is_signed_by(&check.application.identity()) {
        return Err(SignedRequestRefused::NotSigned);
    }
    if stations.stations.keys() != [*check.station] {
        return Err(SignedRequestRefused::AnotherStation);
    }
    if expiry.has_passed(check.now) {
        return Err(SignedRequestRefused::Expired);
    }
    if expiry.lies_beyond(check.now, SIGNED_REQUEST_MAX_LIFE_MS + Expiry::CLOCK_SKEW_MS) {
        return Err(SignedRequestRefused::RunsTooLong);
    }
    Ok(())
}

/// Why a station does not honour a request signed whole. It refuses each as `not_authorized`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SignedRequestRefused {
    #[error("the application's key did not sign the request")]
    NotSigned,
    #[error("the request is signed for another station, or for more than one")]
    AnotherStation,
    #[error("the request's signature has expired")]
    Expired,
    #[error("the request's expiry lies further off than a station honours a signed request for")]
    RunsTooLong,
}
