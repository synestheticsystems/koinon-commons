use crate::stored_forms::enum_stored_by_name;

enum_stored_by_name! {
    /// Where a job's latest attempt stands. Stored as its name in snake case, such as `accepted`, so
    /// that a query reads `state = 'accepted'`. A name this build does not know does not read: a state
    /// added later comes in a component of its own, which a receiver must understand, so that no
    /// receiver reads it as a state it knows.
    pub enum AttemptState {
        /// Recorded, with no outcome yet: the job may be with the device. A job is recorded so before
        /// anything is handed over, and a station that starts and finds a job in this state does not
        /// send it again: it marks the attempt failed as `interrupted`, because nobody knows whether
        /// the job printed.
        Accepted = "accepted",
        /// Handed to the device, or to the system software in front of it. A registry's record says so
        /// once the station has reported it, not when the registry passes the job on. A repeat of the
        /// request is answered with this outcome and prints nothing.
        Sent = "sent",
        /// The latest attempt failed. Sending the job again under its id tries once more, and prints
        /// again whatever the failed attempt had already put on paper.
        Failed = "failed",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ankurah::{property::PropertyError, value::Value, Property};

    // The names a station's journal of Pylon 0.1.0 stores a job's state under.
    #[test]
    fn each_state_is_stored_as_its_name_and_reads_back() {
        let names: Vec<&str> = AttemptState::ALL.iter().map(|state| state.name()).collect();
        assert_eq!(names, ["accepted", "sent", "failed"]);
        for &state in AttemptState::ALL {
            assert_eq!(AttemptState::from_value(state.into_value().unwrap()).unwrap(), state);
            assert_eq!(state.to_string(), state.name());
        }
    }

    #[test]
    fn a_name_this_build_does_not_know_does_not_read() {
        let read = AttemptState::from_value(Some(Value::String("printed".to_owned())));
        assert!(matches!(read, Err(PropertyError::InvalidValue { .. })), "{read:?}");
    }
}
