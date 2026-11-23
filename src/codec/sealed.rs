//! An envelope carried whole inside another, as the bytes it was encoded to: what a signature
//! covers. Whoever carries it on, reading the envelope around it and encoding one of its own,
//! passes these bytes on untouched, so that a signature over them still holds wherever they go.

use ankurah::{property::PropertyError, Property, Value};

use super::{DecodeError, Limits, MintedId, ReceivedEnvelope};

#[cfg(test)]
mod tests;

/// An envelope's bytes, carried as a property of a component of another envelope. A receiver
/// reads them through the carrier's `Signed::open`, which checks the signature against
/// [`SealedEnvelope::as_bytes`] and only then decodes them, so that no receiver can read them
/// first. The envelope's must-understand list concerns whoever reads it, not whoever carries it.
/// Stored, it is a binary value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SealedEnvelope(Vec<u8>);

impl SealedEnvelope {
    /// Bytes that `EnvelopeBuilder::encode` gave, or that arrived as an envelope.
    pub fn new(bytes: Vec<u8>) -> Self {
        SealedEnvelope(bytes)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// The envelope these bytes hold, as `ReceivedEnvelope::decode_own_components` reads one, since
    /// what a key signed is the envelope's own components and no reader of it reads its records:
    /// for a carrier's `Signed::open` alone, which verifies its signature first.
    pub(crate) fn decode(&self, limits: &Limits, understood: &[MintedId]) -> Result<ReceivedEnvelope<'_>, DecodeError> {
        ReceivedEnvelope::decode_own_components(&self.0, limits, understood)
    }
}

impl Property for SealedEnvelope {
    const VALUE_TYPE: &'static str = "binary";

    fn into_value(&self) -> Result<Option<Value>, PropertyError> {
        Ok(Some(Value::Binary(self.0.clone())))
    }

    fn from_value(value: Option<Value>) -> Result<Self, PropertyError> {
        match value {
            Some(Value::Binary(bytes)) => Ok(SealedEnvelope(bytes)),
            Some(other) => Err(PropertyError::InvalidVariant { given: other, ty: "SealedEnvelope".to_owned() }),
            None => Err(PropertyError::Missing),
        }
    }
}
