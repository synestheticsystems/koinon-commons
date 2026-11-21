use std::fmt;

use ankurah::{property::PropertyError, value::Value, Property};

use crate::hex::{from_lower_hex, lower_hex};

/// The SHA-256 of a document's bytes, by which a print names the document that follows its
/// request, before the document arrives. Stored as its 64 lower-case hexadecimal digits; a text of
/// any other shape does not read.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct DocumentHash([u8; 32]);

impl DocumentHash {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        DocumentHash(bytes)
    }

    /// Reads the text [`DocumentHash::text`] writes, and no other.
    pub fn from_text(text: &str) -> Result<Self, NotADocumentHash> {
        from_lower_hex(text).map(DocumentHash).ok_or_else(|| NotADocumentHash(text.to_owned()))
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// The hash's 32 bytes as 64 lower-case hexadecimal digits.
    pub fn text(&self) -> String {
        lower_hex(&self.0)
    }
}

impl fmt::Display for DocumentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text())
    }
}

impl fmt::Debug for DocumentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DocumentHash").field(&self.text()).finish()
    }
}

/// A text that is no [`DocumentHash`]: not 64 lower-case hexadecimal digits. It holds the text.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0:?} is no SHA-256 in 64 lower-case hexadecimal digits")]
pub struct NotADocumentHash(pub String);

impl Property for DocumentHash {
    const VALUE_TYPE: &'static str = "string";

    fn into_value(&self) -> Result<Option<Value>, PropertyError> {
        Ok(Some(Value::String(self.text())))
    }

    fn from_value(value: Option<Value>) -> Result<Self, PropertyError> {
        match value {
            Some(Value::String(text)) => DocumentHash::from_text(&text)
                .map_err(|refused| PropertyError::InvalidValue { value: refused.0, ty: "DocumentHash".to_owned() }),
            Some(other) => Err(PropertyError::InvalidVariant { given: other, ty: "DocumentHash".to_owned() }),
            None => Err(PropertyError::Missing),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hash_is_stored_as_its_lower_case_hexadecimal_digits() {
        let hash = DocumentHash::from_bytes([0xab; 32]);
        let stored = hash.into_value().unwrap();
        assert_eq!(stored, Some(Value::String("ab".repeat(32))));
        assert_eq!(DocumentHash::from_value(stored).unwrap(), hash);
    }

    #[test]
    fn a_text_of_another_shape_does_not_read() {
        for refused in ["AB".repeat(32), "ab".repeat(31), "zz".repeat(32)] {
            assert_eq!(DocumentHash::from_text(&refused), Err(NotADocumentHash(refused.clone())));
            let read = DocumentHash::from_value(Some(Value::String(refused.clone())));
            assert!(matches!(read, Err(PropertyError::InvalidValue { .. })), "{refused}: {read:?}");
        }
    }
}
