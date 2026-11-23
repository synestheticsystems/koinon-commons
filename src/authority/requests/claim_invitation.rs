use ankurah::Model;

use crate::codec::Minted;

/// Claims an invitation that a station's person made, which pairs the computer whose key the
/// connection proves with the station directly: the station then shares the devices the
/// invitation offers with that key, as an application that no registry keeps, whose signature is
/// the authority of the computer's own requests (`SignedRequest`). It requires `InvitationClaim`,
/// whose secret is the claim's whole authority.
///
/// It is answered with the station's own `Station` record, whose name the computer shows its
/// person, and a record for each device the invitation offers, as a `ListDevices` is answered, so
/// that the computer knows what it may use and where with no registry to ask. The station takes a
/// claim whose secret is that of an invitation of its own that is unclaimed, uncancelled and
/// unexpired, from a key it does not share its devices with already, by either way, and that has
/// not retired itself, by a retirement the station keeps on any record of the key, open or closed,
/// since a station never shares with a retired key again (`Retirement`); and it takes such a claim
/// even while it shares its devices with no application. It refuses as `not_authorized` a claim
/// whose secret matches no invitation, as it refuses any authority it does not honour, and one
/// whose invitation was claimed already, cancelled or let lapse, with words that say which, since
/// only a holder of the secret can learn it: an invitation grants what it offers once. It refuses
/// as `not_authorized` too a claim from a key it shares with already, or one that retired itself,
/// and leaves the invitation unclaimed, so that its person can still pass it to the computer it was
/// for.
///
/// A sender lists it as a component a receiver must understand, so that a receiver that does not
/// know it refuses it by name.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "authority.claim_invitation")]
#[model(id = "ab8nnee5WtDPOdO6TCC1vO96H0P14mglv6KwGmQtgYk")]
#[model(no_ffi)]
pub struct ClaimInvitation {}
