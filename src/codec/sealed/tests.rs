//! An envelope carried sealed inside another: its bytes reach their reader as they were written,
//! through a carrier that reads the envelope around them and writes one of its own, so that a
//! signature over them, checked before they are decoded, still holds. SHA-256 over the bytes
//! stands in for the signature.

use ankurah::Model;
use sha2::{Digest, Sha256};

use super::*;
use crate::codec::{
    tests::{decoded, label, one, pointer, property, Deadline, Label},
    EnvelopeBuilder, Minted,
};

/// An envelope and what vouches for its bytes, as a signed request carries them.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "tests.signed")]
#[model(id = "CAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAg")]
#[model(no_ffi)]
pub struct Signed {
    #[property(id = "gYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYE")]
    pub content: SealedEnvelope,
    #[property(id = "goKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoI")]
    pub signature: Vec<u8>,
}

/// What stands in here for a signature over `bytes`: their SHA-256.
fn signature(bytes: &[u8]) -> Vec<u8> {
    Sha256::digest(bytes).to_vec()
}

/// An envelope that holds `content` sealed, beside its signature.
fn carrying(content: &[u8]) -> Vec<u8> {
    one(&Signed { content: SealedEnvelope::new(content.to_vec()), signature: signature(content) })
}

/// An envelope of a Label and of a Deadline that its reader must understand.
fn restricted() -> Vec<u8> {
    let mut envelope = EnvelopeBuilder::new();
    envelope.add(&label()).unwrap().add_must_understand(&Deadline { until: 1 }).unwrap();
    envelope.encode()
}

#[test]
fn a_sealed_envelope_is_stored_as_a_binary_value_of_its_bytes() {
    let sealed = SealedEnvelope::new(restricted());
    let stored = sealed.into_value().unwrap();
    assert_eq!(stored, Some(Value::Binary(restricted())));
    assert_eq!(SealedEnvelope::from_value(stored).unwrap(), sealed);
    assert!(matches!(SealedEnvelope::from_value(Some(Value::String("sealed".into()))), Err(PropertyError::InvalidVariant { .. })));
    assert!(matches!(SealedEnvelope::from_value(None), Err(PropertyError::Missing)));
}

#[test]
fn a_signature_is_checked_over_the_sealed_bytes_as_they_arrived_and_only_then_are_they_decoded() {
    let content = restricted();
    let bytes = carrying(&content);
    let signed: Signed = decoded(&bytes).components().read().unwrap();
    assert_eq!(signed.content.as_bytes(), content);
    assert_eq!(signature(signed.content.as_bytes()), signed.signature);
    let opened = signed.content.decode(&Limits::default(), &[Deadline::ID]).unwrap();
    assert_eq!(opened.components().read::<(Label, Deadline)>().unwrap(), (label(), Deadline { until: 1 }));
}

// Whoever carries a sealed envelope reads only the envelope around it.
#[test]
fn a_sealed_envelopes_must_understand_list_concerns_its_reader_and_not_its_carrier() {
    let bytes = carrying(&restricted());
    let signed: Signed = ReceivedEnvelope::decode(&bytes, &Limits::default(), &[]).unwrap().components().read().unwrap();
    let refused = signed.content.decode(&Limits::default(), &[]).unwrap_err();
    assert_eq!(refused, DecodeError::NotUnderstood { components: vec![Deadline::ID] });
    assert!(signed.content.decode(&Limits::default(), &[Deadline::ID]).is_ok());
}

/// A Label as another encoder may write it, with its properties in the reverse order of their ids
/// and the head of their map in two bytes where one would do: an envelope this codec reads and
/// never writes.
fn written_elsewhere() -> (Label, Vec<u8>) {
    let id = |minted: MintedId| [&[0x58, 0x20][..], minted.as_bytes()].concat();
    let bytes = [
        [vec![0x83, 0x80, 0xa1], id(Label::ID), vec![0xb8, 0x03]].concat(),
        [id(property::<Label>("note")), vec![0x67], b"Fragile".to_vec()].concat(),
        [id(property::<Label>("copies")), vec![0x01]].concat(),
        [id(property::<Label>("name")), vec![0x63], b"4x6".to_vec()].concat(),
        vec![0x80],
    ]
    .concat();
    (Label { name: "4x6".into(), copies: 1, note: Some("Fragile".into()) }, bytes)
}

// The carrier reads the envelope it was sent and writes one of its own around the same sealed
// bytes, with a component of its own added. Bytes this codec would write otherwise show that
// nothing on the way decodes and encodes them again.
#[test]
fn a_carrier_that_writes_an_envelope_of_its_own_passes_the_sealed_bytes_on_as_they_were_written() {
    let (written, content) = written_elsewhere();
    assert_ne!(content, one(&written));
    let sent = carrying(&content);
    let signed: Signed = decoded(&sent).components().read().unwrap();
    let passed_on = EnvelopeBuilder::new().add(&signed).unwrap().add(&pointer()).unwrap().encode();
    assert_ne!(passed_on, sent);

    let signed: Signed = decoded(&passed_on).components().read().unwrap();
    assert_eq!(signed.content.as_bytes(), content);
    assert_eq!(signature(signed.content.as_bytes()), signed.signature);
    assert_eq!(signed.content.decode(&Limits::default(), &[]).unwrap().components().read::<Label>().unwrap(), written);
}

// To the envelope that carries them, sealed bytes are one byte string: their reader reads them as
// an envelope only when it decodes them, after the signature over them holds, and within its own
// limits.
#[test]
fn sealed_bytes_are_read_as_an_envelope_only_when_their_reader_decodes_them_and_within_its_limits() {
    let cut_short = [0x83, 0x80];
    let signed: Signed = decoded(&carrying(&cut_short)).components().read().unwrap();
    assert_eq!(signed.content.decode(&Limits::default(), &[]).unwrap_err(), DecodeError::Truncated { at: 2 });

    let signed: Signed = decoded(&carrying(&restricted())).components().read().unwrap();
    let tight = Limits { max_depth: 3, ..Limits::default() };
    assert!(matches!(signed.content.decode(&tight, &[Deadline::ID]), Err(DecodeError::TooDeep { limit: 3, .. })));
}
