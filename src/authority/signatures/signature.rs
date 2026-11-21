use std::fmt;

use ankurah::{property::PropertyError, value::Value, Property};

use crate::hex::lower_hex;

/// An Ed25519 signature, 64 bytes, as a carrier holds its signer's over the envelope it carries.
/// Stored and carried as a binary value; one of any other length is no signature.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Signature([u8; 64]);

impl Signature {
    pub fn from_bytes(bytes: [u8; 64]) -> Self {
        Signature(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }
}

impl From<ed25519_dalek::Signature> for Signature {
    fn from(signature: ed25519_dalek::Signature) -> Self {
        Signature(signature.to_bytes())
    }
}

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Signature").field(&lower_hex(&self.0)).finish()
    }
}

impl Property for Signature {
    const VALUE_TYPE: &'static str = "binary";

    fn into_value(&self) -> Result<Option<Value>, PropertyError> {
        Ok(Some(Value::Binary(self.0.to_vec())))
    }

    fn from_value(value: Option<Value>) -> Result<Self, PropertyError> {
        match value {
            Some(Value::Binary(bytes)) => <[u8; 64]>::try_from(bytes.as_slice())
                .map(Signature)
                .map_err(|_| PropertyError::InvalidValue { value: format!("{} bytes", bytes.len()), ty: "Signature".to_owned() }),
            Some(other) => Err(PropertyError::InvalidVariant { given: other, ty: "Signature".to_owned() }),
            None => Err(PropertyError::Missing),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_signature_is_stored_as_its_64_bytes() {
        let signature = Signature::from_bytes([7; 64]);
        let stored = signature.into_value().unwrap();
        assert_eq!(stored, Some(Value::Binary(vec![7; 64])));
        assert_eq!(Signature::from_value(stored).unwrap(), signature);
    }

    #[test]
    fn bytes_of_another_length_are_no_signature() {
        for len in [0, 63, 65] {
            let read = Signature::from_value(Some(Value::Binary(vec![7; len])));
            assert!(matches!(read, Err(PropertyError::InvalidValue { .. })), "{len}: {read:?}");
        }
    }
}
