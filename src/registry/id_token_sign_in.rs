use std::fmt;

use ankurah::Model;

use crate::codec::Minted;

/// A person's sign-in, as a station hands it to a registry so that the registry keeps that person
/// signed in on the connection for a registration: the ID token the identity provider issued to
/// the station, and the seed whose nonce ties the token to this sign-in. The registry checks the
/// provider's signature over the token's own text, and takes the token only as one made for the
/// station's sign-in to it, so that a token that leaks with its seed registers no other station,
/// and no other registry takes it:
///
/// - a token whose provider's claim of the registry names another registry is refused, whatever
///   its nonce;
/// - a token whose claim names this registry is taken with the nonce made from the station's key
///   and the seed, or with the one made from this registry's key as well;
/// - a token without the claim is taken with the nonce made from this registry's key, the
///   station's and the seed alone.
///
/// A token the registry does not take is refused as `credential_refused`. Its debug form shows
/// neither the token nor the seed, which together would sign the person in.
// OPEN: whether a person's sign-in, one node signing a person in to another, belongs beside the
// nodes' `NodeLink` rather than in `registry`, whose name every label here holds for good. Lean:
// `registry`, since only a registry takes a sign-in.
#[derive(Model, Minted, Clone, PartialEq)]
#[model(label = "registry.id_token_sign_in")]
#[model(id = "ck1lQOJsZpHeEfj55BrumcImCMNzi03jUCbrBJ6LaZo")]
#[model(no_ffi)]
pub struct IdTokenSignIn {
    /// The ID token, as the provider's token endpoint gave it.
    #[active_type(LWW)]
    #[property(id = "vUwIVj80PS3IjUEJL6OfqPomjzwruL5w9OGsTXZwVYs")]
    pub id_token: String,
    /// The seed the nonce of the station's authorization request was made from. The station draws
    /// one for each sign-in and hands it to the registry alone.
    #[active_type(LWW)]
    #[property(id = "FpArjlvlXMTyVFAylAONByjR4ohZUkbrkesFvZR5IlI")]
    pub nonce_seed: String,
}

impl fmt::Debug for IdTokenSignIn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IdTokenSignIn").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A log that formats a sign-in holds nothing that would sign the person in.
    #[test]
    fn a_sign_in_formats_without_its_token_or_its_seed() {
        let sign_in = IdTokenSignIn {
            id_token: "eyJhbGciOiJSUzI1NiJ9.eyJzdWIiOiIxIn0.c2ln".into(),
            nonce_seed: "kV3pQ8sN1xR6tB0yW4zH7mC2fJ9dL5gA8eU3oI1nT6c".into(),
        };
        assert_eq!(format!("{sign_in:?}"), "IdTokenSignIn { .. }");
    }
}
