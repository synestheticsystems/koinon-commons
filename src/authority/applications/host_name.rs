use std::fmt;

use ankurah::{property::PropertyError, value::Value, Property};

/// A host name in the one form in which a registry and a station write it, so that two writings of
/// one address are the same text: the form the host parser of an HTTPS address gives, the WHATWG URL
/// standard's, which lowers the case and writes an internationalized name in its ASCII form
/// (`xn--`). Its labels are joined by dots, each of 1 to 63 lower-case ASCII letters, digits and
/// hyphens, none of which begins or ends with a hyphen, and it has at most 253 characters in all,
/// such as `acme.example.com`.
///
/// An IP address is no host name here, however it is written, since an application's people log in
/// at a name its certificate gives: the parser reads `127.1` as the address `127.0.0.1`, so a text
/// that the parser reads as an address of any spelling is refused, rather than taken as one of
/// several writings of it. A scheme, a port, a path, a trailing dot, an empty label, or a label the
/// parser's ASCII form refuses, such as `xn--a`, makes a text no host name. Stored and carried as its
/// text.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HostName(String);

impl HostName {
    /// The most characters a host name has in all.
    const MAX_LEN: usize = 253;
    /// The most characters one of its labels has.
    const MAX_LABEL_LEN: usize = 63;

    /// `text`, which is already in the one form, as a registry signs it and a receiver reads it: a
    /// text the parser writes any other way is no host name, rather than another writing of one.
    pub fn new(text: impl Into<String>) -> Result<Self, NotAHostName> {
        let text = text.into();
        match parsed(&text) {
            Some(parsed) if parsed == text => Ok(HostName(text)),
            _ => Err(NotAHostName(text)),
        }
    }

    /// The host name a person typed, in the one form, as the parser writes it: how a station writes
    /// the address its person typed before it compares it with the one a registry's endorsement
    /// names. `Acme.Example.com` is `acme.example.com`, and `bücher.example` is
    /// `xn--bcher-kva.example`.
    pub fn from_typed(typed: &str) -> Result<Self, NotAHostName> {
        parsed(typed).map(HostName).ok_or_else(|| NotAHostName(typed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// `text` as the host parser of an HTTPS address writes it, when the parser reads it as a name and
/// that name keeps the rules of [`HostName`]'s labels; none when it reads an IP address, or nothing.
fn parsed(text: &str) -> Option<String> {
    let url::Host::Domain(name) = url::Host::parse(text).ok()? else { return None };
    let label = |label: &str| {
        (1..=HostName::MAX_LABEL_LEN).contains(&label.len())
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    };
    (name.len() <= HostName::MAX_LEN && name.split('.').all(label)).then_some(name)
}

impl fmt::Display for HostName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A text that is no [`HostName`]: not a host name alone, an IP address, or not in the one form a
/// host name is written in. It holds the text.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0:?} is no host name in the form the host parser of an HTTPS address writes, with no scheme, port or path")]
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
        for text in ["acme.example.com", "xn--bcher-kva.example", "localhost", "label-printer-2.acme.example", "123.example"] {
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

    // Each is an address as a person might type it, one written another way, an IP address in one
    // of its spellings, or a name the parser's ASCII form refuses: none is a text a station could
    // compare with the address its person typed, and each is refused where a property reads it, as
    // a signed endorsement's address is read.
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
            "XN--BCHER-KVA.example",
            "a%41.example",
            "acme..example.com",
            "-acme.example.com",
            "acme-.example.com",
            "acme_labels.example.com",
            " acme.example.com",
            "127.1",
            "127.0.0.1",
            "0x7f.1",
            "[::1]",
            "example.123",
            "xn--a.pt",
        ] {
            assert_eq!(HostName::new(refused), Err(NotAHostName(refused.to_owned())), "{refused:?}");
            let read = HostName::from_value(Some(Value::String(refused.to_owned())));
            assert!(matches!(&read, Err(PropertyError::InvalidValue { value, .. }) if value == refused), "{refused:?}: {read:?}");
        }
    }

    // The parser that writes a typed address is the one a received address is held to, so the two
    // compare as texts.
    #[test]
    fn a_typed_address_is_written_in_the_one_form_and_an_ip_address_is_refused() {
        for (typed, written) in [("Acme.Example.COM", "acme.example.com"), ("bücher.example", "xn--bcher-kva.example")] {
            assert_eq!(HostName::from_typed(typed).unwrap(), HostName::new(written).unwrap(), "{typed:?}");
        }
        for refused in ["127.1", "127.0.0.1", "[::1]", "xn--a.pt", "acme.example.com.", "acme.example.com/login"] {
            assert_eq!(HostName::from_typed(refused), Err(NotAHostName(refused.to_owned())), "{refused:?}");
        }
    }
}
