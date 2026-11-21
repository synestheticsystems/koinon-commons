use std::fmt;

use ankurah::{property::PropertyError, value::Value, Property};

/// How many bytes a document has. Stored as an `i64`; a number below zero does not read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DocumentLength(u64);

impl DocumentLength {
    /// A length of `bytes`, which an `i64` holds.
    pub fn new(bytes: u64) -> Result<Self, NotADocumentLength> {
        i64::try_from(bytes).map(|_| DocumentLength(bytes)).map_err(|_| NotADocumentLength(bytes.to_string()))
    }

    pub fn get(self) -> u64 {
        self.0
    }
}

/// A number that is no [`DocumentLength`]: below zero, or beyond what an `i64` holds. It holds
/// the number's text.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0} is no number of bytes a document has")]
pub struct NotADocumentLength(pub String);

impl fmt::Display for DocumentLength {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Property for DocumentLength {
    const VALUE_TYPE: &'static str = "i64";

    fn into_value(&self) -> Result<Option<Value>, PropertyError> {
        Ok(Some(Value::I64(i64::try_from(self.0).expect("a DocumentLength holds no more than an i64 does"))))
    }

    fn from_value(value: Option<Value>) -> Result<Self, PropertyError> {
        match value {
            Some(Value::I64(bytes)) => u64::try_from(bytes).map(DocumentLength).map_err(|_| PropertyError::InvalidValue {
                value: NotADocumentLength(bytes.to_string()).to_string(),
                ty: "DocumentLength".to_owned(),
            }),
            Some(other) => Err(PropertyError::InvalidVariant { given: other, ty: "DocumentLength".to_owned() }),
            None => Err(PropertyError::Missing),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_length_reads_back_as_stored() {
        for bytes in [0, 6, i64::MAX as u64] {
            let length = DocumentLength::new(bytes).unwrap();
            assert_eq!(DocumentLength::from_value(length.into_value().unwrap()).unwrap(), length);
        }
    }

    #[test]
    fn a_length_below_zero_or_beyond_an_i64_does_not_read() {
        assert!(DocumentLength::new(u64::MAX).is_err());
        let read = DocumentLength::from_value(Some(Value::I64(-1)));
        assert!(matches!(read, Err(PropertyError::InvalidValue { .. })), "{read:?}");
    }
}
