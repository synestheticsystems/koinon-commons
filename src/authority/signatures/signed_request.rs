use ankurah::Model;
use ankurah_commons::property::Timestamp;

use super::{Signature, Signed};
use crate::{
    authority::Expiry,
    codec::{Minted, SealedEnvelope},
    public_key::PublicKey,
};

/// A request whose authority is the signature of the application it comes from: the request
/// whole, as the application's key signed it. A registry's gateway sends one for each job a caller
/// sends it, signed with the key of the application the caller acts for, and a process that holds
/// an application's key sends its own requests so, by any path, as a computer paired with a station
/// directly does. Whoever delivers it, it is honoured only at the station it names, and only until
/// it expires.
///
/// Its envelope carries the request component and the facts the request requires, with these,
/// each listed as a component a receiver must understand:
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
/// 2. the request opens ([`Signed::open`]): its signature verifies over the bytes as they arrived,
///    and then its envelope decodes for a reader of signed requests, which understands the
///    requests a station takes signed, their facts and their restrictions, among the envelope's
///    own components;
/// 3. its envelope carries one request component, and no carrier: a `PresentedSlip`, or another
///    `SignedRequest`, inside it is refused as `bad_request`, since a request has its authority
///    from a signature over it or from a slip beside it, never from both;
/// 4. it lists as one a receiver must understand no component but that request component, the
///    facts that request acts on, `AtStations` and `Expiry`: one that no reader of that request
///    acts on, such as a `DocumentFollows` in a weigh, whose station reads no document after it,
///    refuses it as `not_understood`;
/// 5. its `AtStations` names this station and no other;
/// 6. its `Expiry` has not passed, and lies no further off than a station honours
///    ([`SignedRequest::runs_too_long`]);
/// 7. and then the station does what the request component asks, of the devices that `authority`
///    sets out the request reaches, taking every fact it acts on from inside the signature.
///
/// In the envelope around the carrier, a sender lists the carrier as a component a receiver must
/// understand, and nothing else: a station refuses as `bad_request` an envelope that lists any
/// other so there, and ignores every other component there. A print's document follows that
/// envelope on the stream, bound by the signed `PrintJob`'s hash.
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
    /// the station checks it, with the drift of clocks beyond. A signed request is honoured
    /// whoever delivers it, and a weigh or a listing of devices names no job, so its expiry is all
    /// that bounds how long a copy of it can be used; a station refuses one whose expiry lies
    /// further off. For a signed print it is also how long the job's id can come again under the
    /// signature.
    pub const MAX_LIFE_MS: i64 = 10 * 60_000;

    /// Whether `expiry`, a signed request's own, lies further off than a station honours a signed
    /// request for, at `now` by the station's clock: more than [`SignedRequest::MAX_LIFE_MS`] past
    /// now, with [`Expiry::CLOCK_SKEW_MS`] beyond for the drift of clocks.
    pub fn runs_too_long(expiry: &Expiry, now: Timestamp) -> bool {
        expiry.lies_beyond(now, Self::MAX_LIFE_MS + Expiry::CLOCK_SKEW_MS)
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 1_794_600_000_000;

    fn expiring(millis: i64) -> Expiry {
        Expiry { expires: Timestamp::from_unix_millis(millis) }
    }

    #[test]
    fn a_signed_request_runs_too_long_only_past_ten_minutes_and_the_drift_of_clocks() {
        let now = Timestamp::from_unix_millis(NOW);
        let longest = NOW + SignedRequest::MAX_LIFE_MS + Expiry::CLOCK_SKEW_MS;
        assert!(!SignedRequest::runs_too_long(&expiring(longest), now));
        assert!(SignedRequest::runs_too_long(&expiring(longest + 1), now));
        assert!(SignedRequest::runs_too_long(&expiring(i64::MAX), Timestamp::from_unix_millis(i64::MIN)));
    }
}
