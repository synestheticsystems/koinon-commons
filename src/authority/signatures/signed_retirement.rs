use ankurah::Model;

use super::{Signature, Signed};
use crate::{
    codec::{Minted, SealedEnvelope},
    public_key::PublicKey,
};

/// An application's retirement, as its own key signed it, so that it proves itself and may come
/// by any path: in a registry's hello, down a registry's held connection, or from anyone on the
/// direct path. Its envelope carries `Retirement`, listed as a component a receiver must
/// understand, and the application it retires is the signer. The one check a station makes of it,
/// however it came, is that it opens ([`Signed::open`]): that its signature verifies, and that its
/// envelope then decodes for a reader of retirements, which understands `Retirement` and nothing
/// else. A sender lists the carrier as a component a receiver must understand wherever it travels,
/// since a station that dropped it would go on trusting a retired key.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "authority.signed_retirement")]
#[model(id = "DJ4LjLVa0nGR2nuLumsOLAiSUGco6AF5bXvuoBlKFDQ")]
#[model(no_ffi)]
pub struct SignedRetirement {
    /// The retirement's envelope, as the application's key signed it.
    #[property(id = "klDF8JI06zWvHbrWS3KxVGxflkpKkhnVoc1SZiNWRHM")]
    pub content: SealedEnvelope,
    /// The application's key, which the retirement retires.
    #[active_type(LWW)]
    #[property(id = "Oi9wcjPKKbRhnc_G7hKZcwc3ZRpSzsWg0MLDi5X8OfM")]
    pub signer: PublicKey,
    #[property(id = "TPmmksar6Oex0-FUOIcDQU9dxQSInZeLLrFe8rV213g")]
    pub signature: Signature,
}

impl Signed for SignedRetirement {
    fn new(content: SealedEnvelope, signer: PublicKey, signature: Signature) -> Self {
        SignedRetirement { content, signer, signature }
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
