use std::fmt;

use ankurah::{property::PropertyError, value::Value, Property};

/// A host name in the one form in which a registry and a station write it, so that two writings of
/// one address are the same text: labels joined by dots, each of 1 to 63 lower-case ASCII letters,
/// digits and hyphens, none of which begins or ends with a hyphen, and at most 253 characters in
/// all, such as `acme.example.com`. An internationalized name is written in its ASCII form
/// (`xn--`). A scheme, a port, a path, a trailing dot or a capital letter makes a text no host
/// name, rather than another way to write one. Stored and carried as its text.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HostName(String);

impl HostName {
    /// The most characters a host name has in all.
    const MAX_LEN: usize = 253;
    /// The most characters one of its labels has.
    const MAX_LABEL_LEN: usize = 63;

    pub fn new(text: impl Into<String>) -> Result<Self, NotAHostName> {
        let text = text.into();
        if is_host_name(&text) {
            Ok(HostName(text))
        } else {
            Err(NotAHostName(text))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Whether `text` is a host name in the form [`HostName`] keeps.
fn is_host_name(text: &str) -> bool {
    let label = |label: &str| {
        (1..=HostName::MAX_LABEL_LEN).contains(&label.len())
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    };
    text.len() <= HostName::MAX_LEN && text.split('.').all(label)
}

impl fmt::Display for HostName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A text that is no [`HostName`]: not a host name alone, or not in the one form a host name is
/// written in. It holds the text.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0:?} is no host name in lower case, with no scheme, port or path")]
pub struct NotAHostName(pub String);

impl Property for HostName {
    const VALUE_TYPE: &'static str = "string";

    fn into_value(&self) -> Result<Option<Value>, PropertyError> {
        Ok(Some(Value::String(self.0.clone())))
    }

    fn from_value(value: Option<Value>) -> Result<Self, PropertyError> {
        match value {
            Some(Value::String(text)) => {
                HostName::new(text).map_err(|refused| PropertyError::InvalidValue { value: refused.0, ty: "HostName".to_owned() })
            }
            Some(other) => Err(PropertyError::InvalidVariant { given: other, ty: "HostName".to_owned() }),
            None => Err(PropertyError::Missing),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_host_name_is_kept_and_stored_as_its_text() {
        for text in ["acme.example.com", "xn--bcher-kva.example", "localhost", "label-printer-2.acme.example"] {
            let host = HostName::new(text).unwrap();
            assert_eq!(host.as_str(), text);
            let stored = host.into_value().unwrap();
            assert_eq!(stored, Some(Value::String(text.to_owned())));
            assert_eq!(HostName::from_value(stored).unwrap(), host);
        }
    }

    #[test]
    fn a_host_name_takes_labels_of_63_characters_and_253_characters_in_all() {
        let label = "a".repeat(63);
        assert!(HostName::new(format!("{label}.example")).is_ok());
        assert!(HostName::new(format!("{label}a.example")).is_err());
        let ending = |len| format!("{label}.{label}.{label}.{}", "a".repeat(len));
        assert_eq!(ending(61).len(), 253);
        assert!(HostName::new(ending(61)).is_ok());
        assert!(HostName::new(ending(62)).is_err());
    }

    // Each is an address as a person might type it, or one written another way, which a station
    // could not compare with the address its person typed.
    #[test]
    fn an_address_that_is_more_than_a_host_name_or_written_another_way_is_no_host_name_however_it_comes() {
        for refused in [
            "",
            "https://acme.example.com",
            "acme.example.com/login",
            "acme.example.com:443",
            "acme.example.com.",
            "Acme.example.com",
            "bücher.example",
            "acme..example.com",
            "-acme.example.com",
            "acme-.example.com",
            "acme_labels.example.com",
            " acme.example.com",
            "[::1]",
        ] {
            assert_eq!(HostName::new(refused), Err(NotAHostName(refused.to_owned())), "{refused:?}");
            let read = HostName::from_value(Some(Value::String(refused.to_owned())));
            assert!(matches!(&read, Err(PropertyError::InvalidValue { value, .. }) if value == refused), "{refused:?}: {read:?}");
        }
    }
}
