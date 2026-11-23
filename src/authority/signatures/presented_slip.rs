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
/// Its envelope carries `PermissionSlip`, the slip's `Party` and the restrictions that slip's
/// documentation lists, each listed as a component a receiver must understand. A station takes a
/// request under the slip when, in this order:
///
/// 1. the slip's signer is an application the station shares its devices with;
/// 2. the slip opens ([`Signed::open`]): its signature verifies over the bytes as they arrived, and
///    then its envelope decodes for a reader of slips, which understands a slip's body, its party
///    and its restrictions and nothing else, so that any other component the slip lists as one a
///    receiver must understand refuses it;
/// 3. its `Party` is the key the connection proves;
/// 4. it was issued no later than now, allowing for the drift of clocks
///    ([`PermissionSlip::is_issued_after`]), runs no longer than a station honours a slip for
///    ([`PermissionSlip::runs_too_long`]), and was issued no earlier than the moment its registry
///    wrote the station's registration for its application, nor than the moment it last voided
///    the application's slips, each compared exactly ([`PermissionSlip::is_issued_before`]);
/// 5. its `Expiry` has not passed;
/// 6. its `AtStations` names this station, and its `AtDevices`, when it has one, the device the
///    request names;
/// 7. and its `AllowsRequests` lists the request component, or the request is a `ListDevices`,
///    which is answered with the devices of the kinds the slip allows.
///
/// The devices such a request reaches are those that `authority` sets out. A sender lists the
/// carrier as a component a receiver must understand.
///
/// [`PermissionSlip::is_issued_after`]: crate::authority::PermissionSlip::is_issued_after
/// [`PermissionSlip::runs_too_long`]: crate::authority::PermissionSlip::runs_too_long
/// [`PermissionSlip::is_issued_before`]: crate::authority::PermissionSlip::is_issued_before
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
