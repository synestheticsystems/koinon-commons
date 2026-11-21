use std::fmt;

use ankurah::{property::PropertyError, value::Value, Property};

/// The unit a scale reads in, as the scale reports it. Stored as its name, such as `kilograms`. A
/// name this build does not know does not read: a unit added later comes in a component of its
/// own, which its sender lists as one a receiver must understand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WeightUnit {
    Grams,
    Kilograms,
    Ounces,
    Pounds,
}

impl WeightUnit {
    pub const ALL: [WeightUnit; 4] = [WeightUnit::Grams, WeightUnit::Kilograms, WeightUnit::Ounces, WeightUnit::Pounds];

    /// The unit's name, as it is stored.
    pub fn name(self) -> &'static str {
        match self {
            WeightUnit::Grams => "grams",
            WeightUnit::Kilograms => "kilograms",
            WeightUnit::Ounces => "ounces",
            WeightUnit::Pounds => "pounds",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        WeightUnit::ALL.into_iter().find(|unit| unit.name() == name)
    }
}

impl fmt::Display for WeightUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl Property for WeightUnit {
    const VALUE_TYPE: &'static str = "string";

    fn into_value(&self) -> Result<Option<Value>, PropertyError> {
        Ok(Some(Value::String(self.name().to_owned())))
    }

    fn from_value(value: Option<Value>) -> Result<Self, PropertyError> {
        match value {
            Some(Value::String(name)) => {
                WeightUnit::from_name(&name).ok_or(PropertyError::InvalidValue { value: name, ty: "WeightUnit".to_owned() })
            }
            Some(other) => Err(PropertyError::InvalidVariant { given: other, ty: "WeightUnit".to_owned() }),
            None => Err(PropertyError::Missing),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_unit_is_stored_as_its_name_and_reads_back() {
        let names: Vec<&str> = WeightUnit::ALL.iter().map(|unit| unit.name()).collect();
        assert_eq!(names, ["grams", "kilograms", "ounces", "pounds"]);
        for unit in WeightUnit::ALL {
            assert_eq!(WeightUnit::from_value(unit.into_value().unwrap()).unwrap(), unit);
        }
    }

    #[test]
    fn a_name_this_build_does_not_know_does_not_read() {
        let read = WeightUnit::from_value(Some(Value::String("stones".to_owned())));
        assert!(matches!(read, Err(PropertyError::InvalidValue { .. })), "{read:?}");
    }
}
