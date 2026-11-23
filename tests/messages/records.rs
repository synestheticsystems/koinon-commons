//! What the station keeps that its checks consult: the applications it shares its devices with,
//! each with how it came to the station and what its person approved it for; the parties its
//! person blocked; the keys whose retirement it keeps; the devices it offers; and the invitations
//! its person made, with the claim that takes one.

use koinon_commons::{authority::InvitationSecret, devices::DeviceIds};
use sha2::{Digest, Sha256};

use super::{
    checks::{requested, Refused},
    *,
};

/// The station's devices, each with the request that uses it: its printer, which prints, and its
/// scale, which weighs.
pub fn offered() -> [(EntityId, MintedId); 2] {
    [(printer_record(), Print::ID), (scale_record(), Weigh::ID)]
}

/// What a station's person approved an application for, and how it came to the station.
pub enum Approval {
    /// Every device the station offers: an application that came through the registry whose
    /// endpoint's key this is.
    ThroughRegistry(PublicKey),
    /// The devices the person offered a computer they paired with the station directly.
    Paired(DeviceIds),
}

impl Approval {
    /// Whether the application came through the registry whose endpoint's key is `registry`.
    pub fn came_through(&self, registry: &PublicKey) -> bool {
        matches!(self, Approval::ThroughRegistry(came_through) if came_through == registry)
    }
}

/// The station's approval of the application whose key is `signer`, when it shares its devices
/// with it: the application, through its registry, and a computer paired with it, offered the
/// printer alone.
pub fn approval(signer: &PublicKey) -> Option<Approval> {
    if *signer == key_of(&application()) {
        Some(Approval::ThroughRegistry(registry()))
    } else if *signer == key_of(&paired_computer()) {
        Some(Approval::Paired(DeviceIds::new(vec![printer_record().into()]).unwrap()))
    } else {
        None
    }
}

/// Whether the station's person blocked the party whose key is `party_key` under the application
/// whose key is `application_key`: whether the station's record of that party beside that
/// application is closed. The person blocked one party of the application.
pub fn is_blocked(application_key: &PublicKey, party_key: &PublicKey) -> bool {
    *application_key == key_of(&application()) && *party_key == blocked_party()
}

/// Whether the station keeps a retirement of the key `key` on a record of it: the key of an
/// application it shared its devices with until the key retired itself.
pub fn has_retired(key: &PublicKey) -> bool {
    *key == retired_application()
}

/// An invitation the station's person made to pair a computer with the station, as the station
/// keeps it: its secret's SHA-256 alone, the devices it offers, and whether a computer claimed it.
struct Invitation {
    secret_hash: [u8; 32],
    offered: DeviceIds,
    claimed: bool,
}

/// The invitations the station's person made to pair computers with the station directly.
pub struct Invitations(Vec<Invitation>);

impl Invitations {
    /// The invitation the station's person made, offering the printer alone, whose secret is
    /// `secret`.
    pub fn offering_the_printer(secret: &InvitationSecret) -> Self {
        let offered = DeviceIds::new(vec![printer_record().into()]).unwrap();
        Invitations(vec![Invitation { secret_hash: Sha256::digest(secret.as_bytes()).into(), offered, claimed: false }])
    }

    /// Takes the claim `received` carries, from the computer whose key its connection proves,
    /// `computer`: the claim's secret is that of an invitation no computer has claimed, and the
    /// computer's key one the station does not share its devices with already and holds no
    /// retirement of. The invitation is then claimed, and the computer an application that the
    /// station shares the devices it offers with, whose approval this gives. A claim refused for
    /// its key leaves the invitation unclaimed.
    pub fn claim(&mut self, received: &ReceivedEnvelope<'_>, computer: &PublicKey) -> Result<Approval, Refused> {
        requested(received, &[])?;
        let (_, claim) = received.components().read::<(ClaimInvitation, InvitationClaim)>()?;
        let secret_hash: [u8; 32] = Sha256::digest(claim.secret.as_bytes()).into();
        let invitation = self
            .0
            .iter_mut()
            .find(|invitation| invitation.secret_hash == secret_hash)
            .ok_or_else(|| Refused::not_authorized("no invitation of this station has that secret"))?;
        if invitation.claimed {
            return Err(Refused::not_authorized("the invitation was claimed already"));
        }
        if approval(computer).is_some() {
            return Err(Refused::not_authorized("the station shares its devices with this key already"));
        }
        if has_retired(computer) {
            return Err(Refused::not_authorized("this key retired itself"));
        }
        invitation.claimed = true;
        Ok(Approval::Paired(invitation.offered.clone()))
    }
}
