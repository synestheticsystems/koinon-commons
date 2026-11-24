use crate::stored_forms::enum_stored_by_name;

enum_stored_by_name! {
    /// The unit a scale reads in, as the scale reports it. Stored as its name, such as `kilograms`. A
    /// name this build does not know does not read: a unit added later comes in a component of its
    /// own, which a receiver must understand.
    pub enum WeightUnit {
        Grams = "grams",
        Kilograms = "kilograms",
        Ounces = "ounces",
        Pounds = "pounds",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ankurah::{property::PropertyError, value::Value, Property};

    #[test]
    fn each_unit_is_stored_as_its_name_and_reads_back() {
        let names: Vec<&str> = WeightUnit::ALL.iter().map(|unit| unit.name()).collect();
        assert_eq!(names, ["grams", "kilograms", "ounces", "pounds"]);
        for &unit in WeightUnit::ALL {
            assert_eq!(WeightUnit::from_value(unit.into_value().unwrap()).unwrap(), unit);
        }
    }

    #[test]
    fn a_name_this_build_does_not_know_does_not_read() {
        let read = WeightUnit::from_value(Some(Value::String("stones".to_owned())));
        assert!(matches!(read, Err(PropertyError::InvalidValue { .. })), "{read:?}");
    }
}
