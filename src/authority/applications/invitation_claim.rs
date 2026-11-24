use ankurah::{envelope::EnvelopeComponent, Model};

use super::InvitationSecret;
use crate::name::Name;

/// A computer's claim of an invitation to pair with a station directly, which the station's person
/// made and passed on to it: the invitation's one-time secret, the claim's whole authority, and
/// what the computer calls itself. A request that holds it, beside `Request`, claims the
/// invitation, and needs no tag of its own, since this component is only ever a claim. The claim
/// pairs the computer whose key the connection proves with the station: the station then shares
/// the devices the invitation offers with that key, as an application that no registry keeps,
/// whose signature is the authority of the computer's own requests, signed whole.
///
/// It is answered with `Response` and a `DeviceListing` that holds the station's own `Station`
/// row, whose name the computer shows its person, and a row for each device the invitation
/// offers, so that the computer knows what it may use and where with no registry to ask. The
/// station takes a claim whose secret is that of an invitation of its own that is unclaimed,
/// uncancelled and unexpired, from a key it does not share its devices with already, by either
/// way, and that has not retired itself, by a retirement the station keeps on any record of the
/// key, open or closed, since a station never shares with a retired key again (`Retirement`); and
/// it takes such a claim even while it shares its devices with no application. It refuses as
/// `not_authorized` a claim whose secret matches no invitation, as it refuses any authority it
/// does not honour, and one whose invitation was claimed already, cancelled or let lapse, with
/// words that say which, since only a holder of the secret can learn it: an invitation grants what
/// it offers once. It refuses as `not_authorized` too a claim from a key it shares with already,
/// or one that retired itself, and leaves the invitation unclaimed, so that its person can still
/// pass it to the computer it was for.
///
/// A receiver must understand it, so that one that does not know it refuses the request, naming
/// it. Its debug form shows none of the secret.
#[derive(Model, EnvelopeComponent, Debug, Clone, PartialEq)]
#[model(label = "authority.invitation_claim")]
#[model(id = "--iYP-sz_qtMYEzojHFuBqX3D8DPQAWLSh46CE0GCVM")]
#[model(no_ffi)]
#[envelope(must_understand)]
pub struct InvitationClaim {
    #[property(id = "fuko466qUehAQXNMfJ6uUjLdEB2omEOys9PMhmJeyjA")]
    #[envelope(number = 0)]
    pub secret: InvitationSecret,
    /// What the computer calls itself, such as "Front desk Mac": the name under which the station
    /// lists it. It proves nothing, so the station's person was told, as they made the invitation,
    /// that the first computer to claim it gets what it offers.
    #[active_type(LWW)]
    #[property(id = "x41uLdQ5LNMjXAQ46Pb0jgTI0hL2F0o9xlJlmXHTqfk")]
    #[envelope(number = 1)]
    pub name: Name,
}

#[cfg(test)]
mod tests {
    use super::*;

    // A log that formats a claim holds no secret that would claim an invitation still open.
    #[test]
    fn a_claim_formats_without_its_secret() {
        let claim = InvitationClaim { secret: InvitationSecret::from_bytes([0xab; 16]), name: Name::new("Front desk Mac").unwrap() };
        assert_eq!(format!("{claim:?}"), r#"InvitationClaim { secret: InvitationSecret(..), name: Name("Front desk Mac") }"#);
    }
}
