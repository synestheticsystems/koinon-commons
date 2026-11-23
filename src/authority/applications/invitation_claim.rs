use ankurah::Model;

use super::InvitationSecret;
use crate::{codec::Minted, name::Name};

/// A computer's claim of an invitation to pair with a station directly, which the station's person
/// made and passed on to it: the invitation's one-time secret, the claim's whole authority, and
/// what the computer calls itself. A `ClaimInvitation` carries it. Its debug form shows none of the
/// secret.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "authority.invitation_claim")]
#[model(id = "--iYP-sz_qtMYEzojHFuBqX3D8DPQAWLSh46CE0GCVM")]
#[model(no_ffi)]
pub struct InvitationClaim {
    #[property(id = "fuko466qUehAQXNMfJ6uUjLdEB2omEOys9PMhmJeyjA")]
    pub secret: InvitationSecret,
    /// What the computer calls itself, such as "Front desk Mac": the name under which the station
    /// lists it. It proves nothing, so the station's person was told, as they made the invitation,
    /// that the first computer to claim it gets what it offers.
    #[active_type(LWW)]
    #[property(id = "x41uLdQ5LNMjXAQ46Pb0jgTI0hL2F0o9xlJlmXHTqfk")]
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
