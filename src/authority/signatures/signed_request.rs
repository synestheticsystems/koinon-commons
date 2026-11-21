use ankurah::Model;

use super::{Signature, Signed};
use crate::{
    codec::{Minted, SealedEnvelope},
    public_key::PublicKey,
};

/// A request whose authority is the signature of the application it comes from: the request
/// whole, as the application's key signed it. A registry's gateway sends one for each job a caller
/// sends it, signed with the key of the application the caller acts for, and a process that holds
/// an application's key sends its own requests so, by any path. Whoever delivers it, it holds only
/// at the station it names, and only until it expires.
///
/// The envelope it carries holds the request component and the facts the request requires, with
/// these, each listed as a component a receiver must understand:
///
/// - `AtStations`, naming the one station the request is for;
/// - `Expiry`, at most [`SignedRequest::MAX_LIFE_MS`] after the moment the station checks it,
///   with the drift of clocks beyond;
/// - for a print, `DocumentFollows`.
///
/// A station takes a signed request when, in this order:
///
/// 1. the signer is an application the station shares its devices with, and for a registry's
///    gateway job one that came through the registry whose connection brought it;
/// 2. the signature holds ([`Signed::verify`]);
/// 3. the envelope decodes, and the station understands every component it lists as one a
///    receiver must understand;
/// 4. its `AtStations` names this station and no other;
/// 5. its `Expiry` has not passed, and lies no further off than a station honours;
/// 6. and then the station does what the request component asks, taking every fact it acts on
///    from inside the signature.
///
/// In the envelope around the carrier, a sender lists the carrier as a component a receiver must
/// understand, and nothing else: a station refuses as `bad_request` an envelope that lists any
/// other so there, and ignores every other component there.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "authority.signed_request")]
#[model(id = "ofnV2BFHS4o_GvvB4_HUa7p6uRsaJr43ZFkwjFZrKj4")]
#[model(no_ffi)]
pub struct SignedRequest {
    /// The request's envelope, as the application's key signed it.
    #[property(id = "phs374geJ1PxlLJBeSrzFP-asPlXqcVCYrJtSSyk2Bw")]
    pub content: SealedEnvelope,
    /// The application's key.
    #[active_type(LWW)]
    #[property(id = "a63-qr_S5mFRlQ4CqcJsdtbK8xp6ZlJIqUQN6U8ZBHc")]
    pub signer: PublicKey,
    #[property(id = "15ncfneZEtS-q49GMH-XC5lv5HeFXpMMhElj7Y_D5qk")]
    pub signature: Signature,
}

impl SignedRequest {
    /// The longest a signed request is honoured for, in milliseconds: ten minutes past the moment
    /// the station checks it, with the drift of clocks beyond. A signed request holds whoever
    /// delivers it, and a weigh or a listing of devices names no job, so its expiry is all that
    /// bounds how long a copy of it can be used; a station refuses one whose expiry lies further
    /// off. For a signed print it is also how long the job's id can come again under the
    /// signature.
    pub const MAX_LIFE_MS: i64 = 10 * 60_000;
}

impl Signed for SignedRequest {
    fn new(content: SealedEnvelope, signer: PublicKey, signature: Signature) -> Self {
        SignedRequest { content, signer, signature }
    }

    fn content(&self) -> &SealedEnvelope {
        &self.content
    }

    fn signer(&self) -> &PublicKey {
        &self.signer
    }

    fn signature(&self) -> &Signature {
        &self.signature
    }
}
