use ed25519_dalek::{Signer, SigningKey};

use super::Signature;
use crate::{
    codec::{DecodeError, Limits, Minted, MintedId, ReceivedEnvelope, SealedEnvelope},
    public_key::PublicKey,
};

/// What the bytes every carrier's signature covers begin with, before a zero byte, so that bytes
/// a key signs as a carried envelope are never taken for bytes the key signs for another purpose.
const LABEL: &[u8] = b"koinon signed envelope";

/// A component that carries an envelope as its signer signed it: the envelope's exact bytes, the
/// signer's key, and the signature. Whoever carries it on, reading the envelope around it and
/// encoding one of its own, passes the bytes on untouched, so that the signature still verifies
/// wherever they go.
///
/// The signature covers the label `koinon signed envelope`, a zero byte, the carrier's minted
/// id, then the envelope's bytes. The carrier's id is among them, so that bytes signed as one kind
/// of statement, such as a permission slip, never verify as another, such as a signed request, even
/// if their signer left out the component that says which. A receiver reads the envelope with
/// [`Signed::open`], which verifies the signature over the bytes as they arrived before it decodes
/// them, and acts only on what they carry, never on a copy of them encoded again.
pub trait Signed: Minted {
    /// The carrier of `content` as the key `signer` signed it.
    fn new(content: SealedEnvelope, signer: PublicKey, signature: Signature) -> Self;

    /// The envelope's bytes, as its signer encoded them.
    fn content(&self) -> &SealedEnvelope;

    /// The key whose secret half made the signature.
    fn signer(&self) -> &PublicKey;

    fn signature(&self) -> &Signature;

    /// `content` signed with `key`, as the holder of an application's key signs a statement.
    fn sign(content: SealedEnvelope, key: &SigningKey) -> Self {
        let signature = key.sign(&signed_bytes(Self::ID, &content));
        Self::new(content, PublicKey::from(key.verifying_key()), Signature::from(signature))
    }

    /// Whether the signer's key made the signature over the content, under this carrier. The
    /// check is strict: a signature that verifies only by a leniency some checks of Ed25519 allow,
    /// such as one by a key of small order, does not.
    fn verify(&self) -> Result<(), BadSignature> {
        let signature = ed25519_dalek::Signature::from_bytes(self.signature().as_bytes());
        self.signer().verifying_key().verify_strict(&signed_bytes(Self::ID, self.content()), &signature).map_err(|_| BadSignature)
    }

    /// The envelope this carries, decoded only once its signature verifies, so that no receiver
    /// reads what a key is said to have signed before it knows that the key signed it. It decodes
    /// the bytes as they arrived, within `limits`, for a reader that understands `understood`: the
    /// components it acts on in this kind of statement, such as a slip's body, its party and its
    /// restrictions, and no other.
    fn open(&self, limits: &Limits, understood: &[MintedId]) -> Result<ReceivedEnvelope<'_>, NotOpened> {
        self.verify()?;
        Ok(self.content().decode(limits, understood)?)
    }
}

/// That a carrier's signature is not its signer's over the envelope it carries: another key made
/// it, the key signed other bytes, or it signed them under another carrier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("the signer's key did not sign this envelope under this carrier")]
pub struct BadSignature;

/// Why a carrier's envelope was not opened: its signature does not verify, or, once it does, its
/// bytes are no envelope that the reader takes.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotOpened {
    #[error(transparent)]
    BadSignature(#[from] BadSignature),
    #[error("the signed envelope does not read: {0}")]
    Unreadable(#[from] DecodeError),
}

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

    fn retirement() -> Retirement {
        Retirement { retired: Timestamp::from_unix_millis(1_794_600_000_000) }
    }

    /// An application's retirement, encoded as an envelope.
    fn retiring() -> SealedEnvelope {
        let mut envelope = EnvelopeBuilder::new();
        envelope.add_must_understand(&retirement()).unwrap();
        SealedEnvelope::new(envelope.encode())
    }

    // What the documentation sets out, worked out apart from `signed_bytes`, so that a signer or a
    // receiver written in another language can rely on it.
    #[test]
    fn the_signature_covers_the_label_a_zero_byte_the_carriers_id_and_the_envelope() {
        let carried = SignedRetirement::sign(retiring(), &application());
        let covered = [b"koinon signed envelope".as_slice(), &[0], SignedRetirement::ID.as_bytes(), retiring().as_bytes()].concat();
        let signature = ed25519_dalek::Signature::from_bytes(carried.signature.as_bytes());
        application().verifying_key().verify_strict(&covered, &signature).unwrap();
        assert_eq!(carried.signer, PublicKey::from(application().verifying_key()));
        assert_eq!(carried.verify(), Ok(()));
    }

    #[test]
    fn a_signature_verifies_over_no_other_bytes_by_no_other_key_and_under_no_other_carrier() {
        let carried = PresentedSlip::sign(retiring(), &application());
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

    #[test]
    fn a_carrier_opens_to_its_envelope_once_its_signature_verifies() {
        let carried = SignedRetirement::sign(retiring(), &application());
        let opened = carried.open(&Limits::default(), &[Retirement::ID]).unwrap();
        assert_eq!(opened.components().read::<Retirement>().unwrap(), retirement());
    }

    // Bytes signed by no one decode no envelope, even bytes that would decode: the signature is
    // verified first.
    #[test]
    fn a_carrier_whose_signature_does_not_verify_opens_nothing_even_over_bytes_that_decode() {
        let carried = SignedRetirement::sign(retiring(), &application());
        let other_key = PublicKey::from(SigningKey::from_bytes(&[2; 32]).verifying_key());
        let forged = SignedRetirement::new(carried.content.clone(), other_key, carried.signature);
        assert!(forged.content.decode(&Limits::default(), &[Retirement::ID]).is_ok(), "the bytes decode as an envelope");
        assert_eq!(forged.open(&Limits::default(), &[Retirement::ID]).unwrap_err(), NotOpened::BadSignature(BadSignature));
    }

    // Signed bytes are still read under the reader's own rules: a component the signer lists as one a
    // receiver must understand, which this reader does not act on, refuses them.
    #[test]
    fn a_carrier_whose_envelope_this_reader_does_not_understand_is_refused_naming_the_component() {
        let carried = SignedRetirement::sign(retiring(), &application());
        let refused = carried.open(&Limits::default(), &[]).unwrap_err();
        assert_eq!(refused, NotOpened::Unreadable(DecodeError::NotUnderstood { components: vec![Retirement::ID] }));
    }
}
