use ankurah::Model;

use super::{Signature, Signed};
use crate::{
    codec::{Minted, SealedEnvelope},
    public_key::PublicKey,
};

/// The permission slip a party's request comes under, as the application's key signed it. The
/// party keeps it whole, as the slip's issuer handed it over, and presents it beside each request
/// it sends a station on the direct path; the party's connection to the station proves the party's
/// key, so no one else can use the slip, and the request itself is not signed.
///
/// The envelope it carries holds `PermissionSlip` and the restrictions that slip's documentation
/// lists, each listed as a component a receiver must understand. A station takes a request under
/// the slip when, in this order:
///
/// 1. the slip's signer is an application the station shares its devices with;
/// 2. the signature holds ([`Signed::verify`]);
/// 3. the envelope decodes, and the station understands every component it lists as one a
///    receiver must understand;
/// 4. the slip's party is the key the connection proves;
/// 5. the slip was issued since the station's person last said yes to the application and since
///    the application's slips were last voided, no later than now, and expires no more than
///    `PermissionSlip::MAX_LIFE_MS` after its issue;
/// 6. its `Expiry` has not passed;
/// 7. its `AtStations` names this station, and its `AtDevices`, when it has one, the device the
///    request names;
/// 8. and its `AllowsRequests` lists the request component.
///
/// A sender lists the carrier as a component a receiver must understand.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "authority.presented_slip")]
#[model(id = "yk-eP-hkf0aNCxmxLmpzYtQoJytOrkndpPNRSzts12g")]
#[model(no_ffi)]
pub struct PresentedSlip {
    /// The slip's envelope, as the application's key signed it.
    #[property(id = "YaEQDFhDGVtxCSB47DgKQpEc0jJCQYQIPtkOirE_73c")]
    pub content: SealedEnvelope,
    /// The application's key.
    #[active_type(LWW)]
    #[property(id = "1J8GPFQ9G0l9wEBhZkRsRZF0AC22wnaD1GsoRQxyTlw")]
    pub signer: PublicKey,
    #[property(id = "D0-9RrU0VMEvabn-_SqBMSVZNM9FkAhi8jaQN-a1p7E")]
    pub signature: Signature,
}

impl Signed for PresentedSlip {
    fn new(content: SealedEnvelope, signer: PublicKey, signature: Signature) -> Self {
        PresentedSlip { content, signer, signature }
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
