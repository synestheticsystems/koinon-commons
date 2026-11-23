//! The names people give what they make, by which others tell it from the rest in a list: a
//! scope, the party of a permission slip, an application, or a computer paired with a station.

use std::fmt;

use ankurah::{property::PropertyError, value::Value, Property};

/// A name a person gives something, such as "Acme" for a scope or "Warehouse label PC" for the
/// party of a slip: text with at least one character that is not white space, kept as it was
/// given, so that a list never shows an entry with nothing to tell it by. Stored and carried as
/// its text; a text with nothing to show does not read.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name(String);

impl Name {
    pub fn new(text: impl Into<String>) -> Result<Self, BlankName> {
        let text = text.into();
        if text.chars().all(char::is_whitespace) {
            Err(BlankName(text))
        } else {
            Ok(Name(text))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A text that is no [`Name`], because it is empty or white space alone. It holds the text.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0:?} shows nothing to tell it by, so it is no name")]
pub struct BlankName(pub String);

impl Property for Name {
    const VALUE_TYPE: &'static str = "string";

    fn into_value(&self) -> Result<Option<Value>, PropertyError> {
        Ok(Some(Value::String(self.0.clone())))
    }

    fn from_value(value: Option<Value>) -> Result<Self, PropertyError> {
        match value {
            Some(Value::String(text)) => {
                Name::new(text).map_err(|refused| PropertyError::InvalidValue { value: refused.0, ty: "Name".to_owned() })
            }
            Some(other) => Err(PropertyError::InvalidVariant { given: other, ty: "Name".to_owned() }),
            None => Err(PropertyError::Missing),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_is_kept_as_given_and_stored_as_its_text() {
        let name = Name::new(" Acme East ").unwrap();
        assert_eq!(name.as_str(), " Acme East ");
        let stored = name.into_value().unwrap();
        assert_eq!(stored, Some(Value::String(" Acme East ".to_owned())));
        assert_eq!(Name::from_value(stored).unwrap(), name);
    }

    #[test]
    fn text_that_is_empty_or_white_space_alone_is_no_name_however_it_comes() {
        for refused in ["", " ", "\t\n", "\u{3000}"] {
            assert_eq!(Name::new(refused), Err(BlankName(refused.to_owned())), "{refused:?}");
            let read = Name::from_value(Some(Value::String(refused.to_owned())));
            assert!(matches!(&read, Err(PropertyError::InvalidValue { value, .. }) if value == refused), "{refused:?}: {read:?}");
        }
    }
}
