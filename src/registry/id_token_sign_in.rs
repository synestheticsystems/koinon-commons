use ankurah::Model;

use crate::codec::Minted;

/// A person's sign-in, as a station hands it to a registry so that the registry holds that person
/// on the connection for a registration: the ID token the identity provider issued to the station,
/// and the seed whose nonce ties the token to this sign-in. The registry checks the provider's
/// signature over the token's own text, and takes the token only as one made for a sign-in to it,
/// by the nonce it makes again from its key, the station's and the seed; a token whose nonce does
/// not match was made for a sign-in elsewhere, and is refused as `credential_refused`.
// OPEN: whether a person's sign-in, one node signing a person in to another, belongs beside the
// nodes' `NodeLink` rather than in `registry`, whose name every label here holds for good. Lean:
// `registry`, since only a registry takes a sign-in.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
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
