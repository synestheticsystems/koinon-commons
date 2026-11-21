use std::fmt;

use ankurah::{property::PropertyError, value::Value, Property};

/// Whether a scale's reading can be taken as the weight on it. Stored as its name in snake case,
/// such as `in_motion`. A name this build does not know does not read: a status added later comes
/// in a component of its own, which its sender lists as one a receiver must understand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WeightStatus {
    /// The scale has settled, and the value is the weight on it.
    Stable,
    /// The scale has not settled: the value is still changing.
    InMotion,
    /// The scale has settled at zero, with nothing on it.
    AtZero,
    /// The scale reads below zero, as a scale that was tared does when its load is lifted.
    BelowZero,
    /// The load is more than the scale can weigh, so the value says nothing of it.
    OverCapacity,
}

impl WeightStatus {
    pub const ALL: [WeightStatus; 5] =
        [WeightStatus::Stable, WeightStatus::InMotion, WeightStatus::AtZero, WeightStatus::BelowZero, WeightStatus::OverCapacity];

    /// The status's name, as it is stored.
    pub fn name(self) -> &'static str {
        match self {
            WeightStatus::Stable => "stable",
            WeightStatus::InMotion => "in_motion",
            WeightStatus::AtZero => "at_zero",
            WeightStatus::BelowZero => "below_zero",
            WeightStatus::OverCapacity => "over_capacity",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        WeightStatus::ALL.into_iter().find(|status| status.name() == name)
    }
}

impl fmt::Display for WeightStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl Property for WeightStatus {
    const VALUE_TYPE: &'static str = "string";

    fn into_value(&self) -> Result<Option<Value>, PropertyError> {
        Ok(Some(Value::String(self.name().to_owned())))
    }

    fn from_value(value: Option<Value>) -> Result<Self, PropertyError> {
        match value {
            Some(Value::String(name)) => {
                WeightStatus::from_name(&name).ok_or(PropertyError::InvalidValue { value: name, ty: "WeightStatus".to_owned() })
            }
            Some(other) => Err(PropertyError::InvalidVariant { given: other, ty: "WeightStatus".to_owned() }),
            None => Err(PropertyError::Missing),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_status_is_stored_as_its_name_and_reads_back() {
        let names: Vec<&str> = WeightStatus::ALL.iter().map(|status| status.name()).collect();
        assert_eq!(names, ["stable", "in_motion", "at_zero", "below_zero", "over_capacity"]);
        for status in WeightStatus::ALL {
            assert_eq!(WeightStatus::from_value(status.into_value().unwrap()).unwrap(), status);
        }
    }

    #[test]
    fn a_name_this_build_does_not_know_does_not_read() {
        let read = WeightStatus::from_value(Some(Value::String("settling".to_owned())));
        assert!(matches!(read, Err(PropertyError::InvalidValue { .. })), "{read:?}");
    }
}
