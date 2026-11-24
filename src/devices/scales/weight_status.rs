use crate::stored_forms::enum_stored_by_name;

enum_stored_by_name! {
    /// Whether a scale's reading can be taken as the weight on it. Stored as its name in snake case,
    /// such as `in_motion`. A name this build does not know does not read: a status added later comes
    /// in a component of its own, which a receiver must understand.
    pub enum WeightStatus {
        /// The scale has settled, and the value is the weight on it.
        Stable = "stable",
        /// The scale has not settled: the value is still changing.
        InMotion = "in_motion",
        /// The scale has settled at zero, with nothing on it.
        AtZero = "at_zero",
        /// The scale reads below zero, as a scale that was tared does when its load is lifted.
        BelowZero = "below_zero",
        /// The load is more than the scale can weigh, so the value says nothing of it.
        OverCapacity = "over_capacity",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ankurah::{property::PropertyError, value::Value, Property};

    #[test]
    fn each_status_is_stored_as_its_name_and_reads_back() {
        let names: Vec<&str> = WeightStatus::ALL.iter().map(|status| status.name()).collect();
        assert_eq!(names, ["stable", "in_motion", "at_zero", "below_zero", "over_capacity"]);
        for &status in WeightStatus::ALL {
            assert_eq!(WeightStatus::from_value(status.into_value().unwrap()).unwrap(), status);
            assert_eq!(status.to_string(), status.name());
        }
    }

    #[test]
    fn a_name_this_build_does_not_know_does_not_read() {
        let read = WeightStatus::from_value(Some(Value::String("settling".to_owned())));
        assert!(matches!(read, Err(PropertyError::InvalidValue { .. })), "{read:?}");
    }
}
