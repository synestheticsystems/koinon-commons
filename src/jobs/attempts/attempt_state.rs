use std::fmt;

use ankurah::{property::PropertyError, value::Value, Property};

/// Where a job's latest attempt stands. Stored as its name in snake case, such as `accepted`, so
/// that a query reads `state = 'accepted'`. A name this build does not know does not read: a state
/// added later comes in a component of its own, which its sender lists as one a receiver must
/// understand, so that no receiver reads it as a state it knows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttemptState {
    /// Recorded, with no outcome yet: the job may be with the device. A job is recorded so before
    /// anything is handed over, and a station that starts and finds a job in this state does not
    /// send it again: it marks the attempt failed as `interrupted`, because nobody knows whether
    /// the job printed.
    Accepted,
    /// Handed to the device, or to the system software in front of it. A registry's record says so
    /// once the station has reported it, not when the registry passes the job on. A repeat of the
    /// request is answered with this outcome and prints nothing.
    Sent,
    /// The latest attempt failed. Sending the job again under its id tries once more, and prints
    /// again whatever the failed attempt had already put on paper.
    Failed,
}

impl AttemptState {
    pub const ALL: [AttemptState; 3] = [AttemptState::Accepted, AttemptState::Sent, AttemptState::Failed];

    /// The state's name, as it is stored.
    pub fn name(self) -> &'static str {
        match self {
            AttemptState::Accepted => "accepted",
            AttemptState::Sent => "sent",
            AttemptState::Failed => "failed",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        AttemptState::ALL.into_iter().find(|state| state.name() == name)
    }
}

impl fmt::Display for AttemptState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl Property for AttemptState {
    const VALUE_TYPE: &'static str = "string";

    fn into_value(&self) -> Result<Option<Value>, PropertyError> {
        Ok(Some(Value::String(self.name().to_owned())))
    }

    fn from_value(value: Option<Value>) -> Result<Self, PropertyError> {
        match value {
            Some(Value::String(name)) => {
                AttemptState::from_name(&name).ok_or(PropertyError::InvalidValue { value: name, ty: "AttemptState".to_owned() })
            }
            Some(other) => Err(PropertyError::InvalidVariant { given: other, ty: "AttemptState".to_owned() }),
            None => Err(PropertyError::Missing),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The names a station's journal of Pylon 0.1.0 stores a job's state under.
    #[test]
    fn each_state_is_stored_as_its_name_and_reads_back() {
        let names: Vec<&str> = AttemptState::ALL.iter().map(|state| state.name()).collect();
        assert_eq!(names, ["accepted", "sent", "failed"]);
        for state in AttemptState::ALL {
            assert_eq!(AttemptState::from_value(state.into_value().unwrap()).unwrap(), state);
        }
    }

    #[test]
    fn a_name_this_build_does_not_know_does_not_read() {
        let read = AttemptState::from_value(Some(Value::String("printed".to_owned())));
        assert!(matches!(read, Err(PropertyError::InvalidValue { .. })), "{read:?}");
    }
}
