use std::{fmt, num::NonZeroU32};

use ankurah::{property::PropertyError, value::Value, Property};
use serde::{Deserialize, Serialize};

/// How many copies a print makes: at least one, and fewer than 2^32. Stored as an `i64`; a
/// number outside that range does not read, so that no sender can ask for none, or for less than
/// none. In JSON, as `PrintJob` derives it, a number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "i64", into = "i64")]
pub struct Copies(NonZeroU32);

impl Copies {
    pub fn new(copies: u32) -> Result<Self, NotCopies> {
        NonZeroU32::new(copies).map(Copies).ok_or(NotCopies(i64::from(copies)))
    }

    pub fn get(self) -> u32 {
        self.0.get()
    }
}

/// A number of copies that is no [`Copies`]: none, less than none, or 2^32 or more.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("{0} is no number of copies a print makes")]
pub struct NotCopies(pub i64);

impl TryFrom<i64> for Copies {
    type Error = NotCopies;

    fn try_from(copies: i64) -> Result<Self, NotCopies> {
        u32::try_from(copies).ok().and_then(|copies| Copies::new(copies).ok()).ok_or(NotCopies(copies))
    }
}

impl From<Copies> for i64 {
    fn from(copies: Copies) -> Self {
        i64::from(copies.get())
    }
}

impl fmt::Display for Copies {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Property for Copies {
    const VALUE_TYPE: &'static str = "i64";

    fn into_value(&self) -> Result<Option<Value>, PropertyError> {
        Ok(Some(Value::I64(i64::from(*self))))
    }

    fn from_value(value: Option<Value>) -> Result<Self, PropertyError> {
        match value {
            Some(Value::I64(copies)) => Copies::try_from(copies)
                .map_err(|refused| PropertyError::InvalidValue { value: refused.to_string(), ty: "Copies".to_owned() }),
            Some(other) => Err(PropertyError::InvalidVariant { given: other, ty: "Copies".to_owned() }),
            None => Err(PropertyError::Missing),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_copy_or_more_reads_back_as_stored() {
        for copies in [1, 2, u32::MAX] {
            let copies = Copies::new(copies).unwrap();
            assert_eq!(Copies::from_value(copies.into_value().unwrap()).unwrap(), copies);
        }
    }

    // The JSON form a print job's copies had before they had a type of their own, a number, so that
    // a print job's JSON reads as before.
    #[test]
    fn copies_are_a_json_number_and_one_outside_the_range_does_not_read() {
        assert_eq!(serde_json::to_value(Copies::new(2).unwrap()).unwrap(), serde_json::json!(2));
        assert_eq!(serde_json::from_value::<Copies>(serde_json::json!(2)).unwrap(), Copies::new(2).unwrap());
        for refused in [serde_json::json!(0), serde_json::json!(-5), serde_json::json!(1_u64 << 32)] {
            assert!(serde_json::from_value::<Copies>(refused.clone()).is_err(), "{refused}");
        }
    }

    #[test]
    fn a_number_of_copies_below_one_or_beyond_the_range_does_not_read() {
        assert_eq!(Copies::new(0), Err(NotCopies(0)));
        for refused in [0, -5, i64::from(u32::MAX) + 1, 1 << 62] {
            let read = Copies::from_value(Some(Value::I64(refused)));
            assert!(matches!(read, Err(PropertyError::InvalidValue { .. })), "{refused}: {read:?}");
        }
    }
}
