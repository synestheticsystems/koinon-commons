//! The checks every station makes of a permission slip before it acts on a request under it, kept
//! beside the slip so that every station honours the same slips and refuses the same ones.

use ankurah::{
    envelope::{DecodeError, Envelope, EnvelopeComponent, Limits, ReadError},
    property::Ref,
    EntityId,
};
use ankurah_commons::{property::Timestamp, requests::AuthToken};
use bytes::Bytes;

use super::{Party, PermissionSlip};
use crate::{
    authority::{AllowsRequests, AtDevices, AtStations, Expiry},
    devices::Device,
    public_key::PublicKey,
    replies::Refusal,
};

#[cfg(test)]
mod tests;

/// The token that presents `slip`, a permission slip's envelope as its application signed it, so
/// that a request comes under the slip: its format is `PermissionSlip`'s id, and its credential
/// the envelope's exact bytes, its signatures among them.
pub fn slip_token(slip: &Envelope) -> AuthToken {
    AuthToken { format: PermissionSlip::ID, credential: slip.bytes().to_vec() }
}

/// The slip that `token` presents, decoded within `limits` and not yet checked: a station opens it
/// to learn which of the applications it shares its devices with signed it, then checks it with
/// [`check_slip`]. A token in another format is no slip, and is refused naming the format, never
/// taken for no token at all.
pub fn open_slip(token: AuthToken, limits: &Limits) -> Result<Envelope, SlipRefused> {
    if token.format != PermissionSlip::ID {
        return Err(SlipRefused::OtherFormat(token.format));
    }
    Envelope::decode(Bytes::from(token.credential), limits).map_err(SlipRefused::Undecodable)
}

/// What a station knows when it checks a permission slip, and what the slip must match.
#[derive(Debug, Clone, Copy)]
pub struct SlipCheck<'a> {
    /// The application, among those the station shares its devices with, whose key the slip names
    /// among its signers: the one whose authority the slip claims.
    pub application: &'a PublicKey,
    /// The key the connection that brought the request proves, which must be the slip's party.
    pub party: &'a PublicKey,
    /// The station's own key, which the slip's `AtStations` must name.
    pub station: &'a PublicKey,
    /// The moment from which the station honours the application's slips, by the clock of the
    /// registry the application came through: when that registry wrote the station's registration
    /// for the application, or last voided the application's slips, whichever came later.
    pub honoured_from: Timestamp,
    /// Now, by the station's clock.
    pub now: Timestamp,
}

/// Checks `slip`, as [`open_slip`] gave it, as every station checks a slip before it acts on a
/// request under it, in this order, and gives the slip's components as read:
///
/// 1. the application's key signed it;
/// 2. it reads as a slip: its `PermissionSlip`, `Party`, `AtStations`, `AllowsRequests` and
///    `Expiry`, and its `AtDevices` when it has one, with no other component a receiver must
///    understand;
/// 3. its party is the key the connection proves;
/// 4. it was issued no later than now, allowing for the drift of clocks
///    ([`PermissionSlip::is_issued_after`]), expires no earlier than it was issued
///    ([`PermissionSlip::expires_before_issue`]), runs no longer than a station honours a slip for
///    ([`PermissionSlip::runs_too_long`]), and was issued no earlier than the moment from which the
///    station honours the application's slips, compared exactly
///    ([`PermissionSlip::is_issued_before`]);
/// 5. its `Expiry` has not passed;
/// 6. and its `AtStations` names this station.
///
/// A station then refuses a party its person blocked under the application, which only it knows,
/// and a request for a device or an operation the slip does not cover ([`slip_covers`]).
// The components are spelled out, as every reader spells out the tuple it reads: no alias names a
// tuple of components.
#[allow(clippy::type_complexity)]
pub fn check_slip(
    slip: &Envelope,
    check: &SlipCheck<'_>,
) -> Result<(PermissionSlip, Party, AtStations, Option<AtDevices>, AllowsRequests, Expiry), SlipRefused> {
    if !slip.is_signed_by(&check.application.identity()) {
        return Err(SlipRefused::NotSigned);
    }
    let (body, party, stations, devices, allows, expiry) =
        slip.get::<(PermissionSlip, Party, AtStations, Option<AtDevices>, AllowsRequests, Expiry)>().map_err(SlipRefused::Unreadable)?;
    if party.key != *check.party {
        return Err(SlipRefused::AnotherParty);
    }
    if body.is_issued_after(check.now) {
        return Err(SlipRefused::IssuedLater);
    }
    if body.expires_before_issue(&expiry) {
        return Err(SlipRefused::ExpiresBeforeIssue);
    }
    if body.runs_too_long(&expiry) {
        return Err(SlipRefused::RunsTooLong);
    }
    if body.is_issued_before(check.honoured_from) {
        return Err(SlipRefused::IssuedBeforeHonoured);
    }
    if expiry.has_passed(check.now) {
        return Err(SlipRefused::Expired);
    }
    if !stations.stations.contains(check.station) {
        return Err(SlipRefused::AnotherStation);
    }
    Ok((body, party, stations, devices, allows, expiry))
}

/// Whether a slip, by its `AtDevices` and `AllowsRequests` as [`check_slip`] gave them, covers
/// `device` for the operation whose tag's id is `operation`: the device is one its `AtDevices`
/// names, or any of the station's when it has none, and the operation one it allows. A station
/// lists, in answer to a `DoListDevices`, the devices a slip covers for the operation of their
/// kind: `DoPrint` for a printer, `DoWeigh` for a scale.
pub fn slip_covers(devices: Option<&AtDevices>, allows: &AllowsRequests, operation: EntityId, device: &Ref<Device>) -> bool {
    devices.is_none_or(|narrowed| narrowed.devices.contains(device)) && allows.requests.contains(&operation)
}

/// Why a station does not honour a permission slip. Each is refused under the code
/// [`SlipRefused::code`] gives.
#[derive(Debug, thiserror::Error)]
pub enum SlipRefused {
    #[error("the token is in the format {0}, which is no permission slip's")]
    OtherFormat(EntityId),
    #[error("the slip does not decode: {0}")]
    Undecodable(DecodeError),
    #[error("the application's key did not sign the slip")]
    NotSigned,
    #[error("the slip does not read: {0}")]
    Unreadable(ReadError),
    #[error("the slip is another party's")]
    AnotherParty,
    #[error("the slip says it was issued later than now")]
    IssuedLater,
    #[error("the slip expires before it was issued")]
    ExpiresBeforeIssue,
    #[error("the slip runs on for longer than a station honours a slip for")]
    RunsTooLong,
    #[error("the slip was issued before the moment from which the station honours its application's slips")]
    IssuedBeforeHonoured,
    #[error("the slip has expired")]
    Expired,
    #[error("the slip does not name this station")]
    AnotherStation,
}

impl SlipRefused {
    /// The code of the `Refusal` a station answers with: `not_understood` for a slip that holds a
    /// component a receiver must understand that the station does not, which a `NotUnderstood`
    /// beside the refusal names; `bad_request` for a slip that does not decode, or does not read as
    /// a slip; and `not_authorized` for every other, since the station honours no authority the
    /// request carries.
    pub fn code(&self) -> &'static str {
        match self {
            SlipRefused::Unreadable(ReadError::NotUnderstood { .. }) => Refusal::NOT_UNDERSTOOD,
            SlipRefused::Undecodable(_) | SlipRefused::Unreadable(_) => Refusal::BAD_REQUEST,
            _ => Refusal::NOT_AUTHORIZED,
        }
    }
}
