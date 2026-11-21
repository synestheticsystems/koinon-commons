use ed25519_dalek::{Signer, SigningKey};

use super::Signature;
use crate::{
    codec::{Minted, MintedId, SealedEnvelope},
    public_key::PublicKey,
};

/// What the bytes every carrier's signature covers begin with, before a zero byte, so that bytes
/// a key signs as a carried envelope are never taken for bytes the key signs for another purpose.
const LABEL: &[u8] = b"koinon signed envelope";

/// A component that carries an envelope as its signer signed it: the envelope's exact bytes, the
/// signer's key, and the signature. Whoever carries it on, reading the envelope around it and
/// encoding one of its own, passes the bytes on untouched, so that the signature still holds
/// wherever they go.
///
/// The signature covers the label `koinon signed envelope`, a zero byte, the carrier's minted
/// id, then the envelope's bytes. The carrier's id is among them, so that bytes signed as one kind
/// of statement, such as a permission slip, never hold as another, such as a signed request, even
/// if their signer left out the component that says which. A receiver checks the signature with
/// [`Signed::verify`] over the bytes as they arrived, then decodes them, and acts only on what they
/// hold, never on a copy of them encoded again.
pub trait Signed: Minted {
    /// The carrier of `content` as the key `signer` signed it.
    fn new(content: SealedEnvelope, signer: PublicKey, signature: Signature) -> Self;

    /// The envelope's bytes, as its signer encoded them.
    fn content(&self) -> &SealedEnvelope;

    /// The key whose secret half made the signature.
    fn signer(&self) -> &PublicKey;

    fn signature(&self) -> &Signature;

    /// `content` signed with `key`, as the holder of an application's key signs a statement.
    fn sign(content: SealedEnvelope, key: &SigningKey) -> Self
    where
        Self: Sized,
    {
        let signature = key.sign(&signed_bytes(Self::ID, &content));
        Self::new(content, PublicKey::from(key.verifying_key()), Signature::from(signature))
    }

    /// Whether the signer's key made the signature over the content, under this carrier. The
    /// check is strict: a signature that holds only by a leniency some checks of Ed25519 allow,
    /// such as one by a key of small order, does not.
    fn verify(&self) -> Result<(), BadSignature> {
        let signature = ed25519_dalek::Signature::from_bytes(self.signature().as_bytes());
        self.signer().verifying_key().verify_strict(&signed_bytes(Self::ID, self.content()), &signature).map_err(|_| BadSignature)
    }
}

/// That a carrier's signature is not its signer's over the envelope it carries: another key made
/// it, the key signed other bytes, or it signed them under another carrier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("the signer's key did not sign this envelope under this carrier")]
pub struct BadSignature;

/// The bytes a carrier's signature covers: the label, a zero byte, the carrier's id, then the
/// envelope's bytes.
fn signed_bytes(carrier: MintedId, content: &SealedEnvelope) -> Vec<u8> {
    let content = content.as_bytes();
    let mut bytes = Vec::with_capacity(LABEL.len() + 1 + carrier.as_bytes().len() + content.len());
    bytes.extend_from_slice(LABEL);
    bytes.push(0);
    bytes.extend_from_slice(carrier.as_bytes());
    bytes.extend_from_slice(content);
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        authority::{PresentedSlip, Retirement, SignedRequest, SignedRetirement},
        codec::EnvelopeBuilder,
    };
    use ankurah_commons::property::Timestamp;

    fn application() -> SigningKey {
        SigningKey::from_bytes(&[1; 32])
    }

    /// An application's retirement, encoded as an envelope.
    fn retirement() -> SealedEnvelope {
        let mut envelope = EnvelopeBuilder::new();
        envelope.add_must_understand(&Retirement { retired: Timestamp::from_unix_millis(1_794_600_000_000) }).unwrap();
        SealedEnvelope::new(envelope.encode())
    }

    // What the documentation sets out, worked out apart from `signed_bytes`, so that a signer or a
    // receiver written in another language can rely on it.
    #[test]
    fn the_signature_covers_the_label_a_zero_byte_the_carriers_id_and_the_envelope() {
        let carried = SignedRetirement::sign(retirement(), &application());
        let covered = [b"koinon signed envelope".as_slice(), &[0], SignedRetirement::ID.as_bytes(), retirement().as_bytes()].concat();
        let signature = ed25519_dalek::Signature::from_bytes(carried.signature.as_bytes());
        application().verifying_key().verify_strict(&covered, &signature).unwrap();
        assert_eq!(carried.signer, PublicKey::from(application().verifying_key()));
        assert_eq!(carried.verify(), Ok(()));
    }

    #[test]
    fn a_signature_holds_over_no_other_bytes_by_no_other_key_and_under_no_other_carrier() {
        let carried = PresentedSlip::sign(retirement(), &application());
        let mut changed = carried.content.as_bytes().to_vec();
        *changed.last_mut().unwrap() ^= 1;
        let other_key = PublicKey::from(SigningKey::from_bytes(&[2; 32]).verifying_key());
        let refused = [
            PresentedSlip::new(SealedEnvelope::new(changed), carried.signer, carried.signature).verify(),
            PresentedSlip::new(carried.content.clone(), other_key, carried.signature).verify(),
            SignedRequest::new(carried.content.clone(), carried.signer, carried.signature).verify(),
        ];
        assert_eq!(refused, [Err(BadSignature); 3]);
    }
}
