use std::fmt;

use ankurah::{property::PropertyError, value::Value, Property};

/// The one-time secret of an invitation to pair a computer with a station directly: 16 bytes the
/// station draws at random when its person makes the invitation. A computer that presents them
/// proves it was handed the invitation, since no one could guess 128 random bits; the station keeps
/// only their SHA-256, so that its records reveal no secret that would claim an invitation still
/// open. Carried as a binary value of exactly 16 bytes; bytes of any other length are no secret.
/// Its debug form shows none of it, so that no log holds a secret.
#[derive(Clone, PartialEq, Eq)]
pub struct InvitationSecret([u8; InvitationSecret::LEN]);

impl InvitationSecret {
    /// How many bytes a secret has.
    pub const LEN: usize = 16;

    /// The secret of these bytes, which the station draws from its system's source of randomness.
    pub fn from_bytes(bytes: [u8; InvitationSecret::LEN]) -> Self {
        InvitationSecret(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; InvitationSecret::LEN] {
        &self.0
    }
}

impl fmt::Debug for InvitationSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("InvitationSecret(..)")
    }
}

impl Property for InvitationSecret {
    const VALUE_TYPE: &'static str = "binary";

    fn into_value(&self) -> Result<Option<Value>, PropertyError> {
        Ok(Some(Value::Binary(self.0.to_vec())))
    }

    fn from_value(value: Option<Value>) -> Result<Self, PropertyError> {
        match value {
            // The refusal names the length alone, never the bytes, which may be most of a secret.
            Some(Value::Binary(bytes)) => <[u8; InvitationSecret::LEN]>::try_from(bytes.as_slice())
                .map(InvitationSecret)
                .map_err(|_| PropertyError::InvalidValue { value: format!("{} bytes", bytes.len()), ty: "InvitationSecret".to_owned() }),
            Some(other) => Err(PropertyError::InvalidVariant { given: other, ty: "InvitationSecret".to_owned() }),
            None => Err(PropertyError::Missing),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_secret_is_carried_as_its_16_bytes() {
        let secret = InvitationSecret::from_bytes(*b"0123456789abcdef");
        let carried = secret.into_value().unwrap();
        assert_eq!(carried, Some(Value::Binary(b"0123456789abcdef".to_vec())));
        assert_eq!(InvitationSecret::from_value(carried).unwrap(), secret);
    }

    #[test]
    fn bytes_of_another_length_are_no_secret() {
        for len in [0, 15, 17, 32] {
            let read = InvitationSecret::from_value(Some(Value::Binary(vec![7; len])));
            assert!(matches!(&read, Err(PropertyError::InvalidValue { value, .. }) if *value == format!("{len} bytes")), "{len}: {read:?}");
        }
    }

    #[test]
    fn a_secret_formats_as_none_of_its_bytes() {
        let formatted = format!("{:?}", InvitationSecret::from_bytes(*b"0123456789abcdef"));
        assert_eq!(formatted, "InvitationSecret(..)");
    }
}
