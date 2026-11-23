use std::fmt;

use ankurah::{property::PropertyError, value::Value, Property};

/// The number a scale read, in its reading's unit: finite, and below zero when the scale reads
/// so, as a tared scale does when its load is lifted. A reading of no number, which an envelope
/// could carry as an infinity or a NaN, does not read. Stored as an `f64`.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct WeightValue(f64);

impl WeightValue {
    pub fn new(value: f64) -> Result<Self, NotAWeight> {
        if value.is_finite() {
            Ok(WeightValue(value))
        } else {
            Err(NotAWeight { value })
        }
    }

    pub fn get(self) -> f64 {
        self.0
    }
}

/// A reading that is no [`WeightValue`], because it is no finite number.
#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
#[error("a scale's reading of {value} is no finite number")]
pub struct NotAWeight {
    pub value: f64,
}

impl fmt::Display for WeightValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Property for WeightValue {
    const VALUE_TYPE: &'static str = "f64";

    fn into_value(&self) -> Result<Option<Value>, PropertyError> {
        Ok(Some(Value::F64(self.0)))
    }

    fn from_value(value: Option<Value>) -> Result<Self, PropertyError> {
        match value {
            Some(Value::F64(value)) => WeightValue::new(value)
                .map_err(|refused| PropertyError::InvalidValue { value: refused.value.to_string(), ty: "WeightValue".to_owned() }),
            Some(other) => Err(PropertyError::InvalidVariant { given: other, ty: "WeightValue".to_owned() }),
            None => Err(PropertyError::Missing),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_finite_reading_reads_back_below_zero_included() {
        for value in [1.25, 0.0, -0.5] {
            let weight = WeightValue::new(value).unwrap();
            assert_eq!(WeightValue::from_value(weight.into_value().unwrap()).unwrap(), weight);
        }
    }

    #[test]
    fn a_reading_of_no_number_does_not_read() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(WeightValue::new(value).is_err(), "{value}");
            let read = WeightValue::from_value(Some(Value::F64(value)));
            assert!(matches!(read, Err(PropertyError::InvalidValue { .. })), "{value}: {read:?}");
        }
    }
}
