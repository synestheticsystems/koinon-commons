//! An envelope carried whole inside another, as the bytes it was encoded to: what a signature
//! covers. Whoever carries it on, reading the envelope around it and encoding one of its own,
//! passes these bytes on untouched, so that a signature over them still holds wherever they go.

use ankurah::{property::PropertyError, Property, Value};

use super::{DecodeError, Limits, MintedId, ReceivedEnvelope};

#[cfg(test)]
mod tests;

/// An envelope's bytes, carried as a property of a component of another envelope. A receiver
/// checks a signature against [`SealedEnvelope::as_bytes`], and only then decodes them. The
/// envelope's must-understand list concerns whoever reads it, not whoever carries it. Stored, it is
/// a binary value.
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

    /// The envelope these bytes hold, as `ReceivedEnvelope::decode` reads one.
    pub fn decode(&self, limits: &Limits, understood: &[MintedId]) -> Result<ReceivedEnvelope<'_>, DecodeError> {
        ReceivedEnvelope::decode(&self.0, limits, understood)
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
