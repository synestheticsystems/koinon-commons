use std::fmt;

use ankurah::{property::PropertyError, value::Value, Model, Property};
use serde::{Deserialize, Serialize};

/// That the device on this entity prints, as the computer's print system describes it.
/// What it is loaded with is `Media`, on the same entity.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "devices.printer")]
#[model(id = "tdb_BWOhavVLjE6j3fg5AxGEvVYxVpy7vSx3IdvCxzE")]
#[model(no_ffi)]
pub struct Printer {
    /// The name of the driver that the print system uses for it, such as "Zebra ZPL Label
    /// Printer": on macOS what the queue reports as its make and model, on Windows the name
    /// of its driver. Every queue has one, though it may name a generic driver and say
    /// nothing of the printer. A server can work out the printer's own language from it.
    #[active_type(LWW)]
    pub driver: String,
    /// The formats of document the print system accepts for it, as MIME types, in the order
    /// the print system lists them, such as `application/pdf`, or `application/vnd.cups-raw`
    /// for bytes passed to the printer untouched. A caller chooses what to send by them, and a
    /// station refuses a document in a format they do not list.
    #[active_type(LWW)]
    pub document_formats: MimeTypes,
}

/// A list of MIME types: at least one, none twice, each a type and a subtype without
/// parameters, in lower case, of the characters a registered name may hold (RFC 6838), such
/// as `application/pdf`. MIME types compare without regard to case, so the list keeps one
/// spelling of each. Stored as a JSON array of the types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Vec<String>", into = "Vec<String>")]
pub struct MimeTypes(Vec<String>);

impl MimeTypes {
    pub fn new(types: Vec<String>) -> Result<Self, NotMimeTypes> {
        if types.is_empty() {
            return Err(NotMimeTypes::Empty);
        }
        for (index, mime_type) in types.iter().enumerate() {
            if !is_mime_type(mime_type) {
                return Err(NotMimeTypes::NotAType(mime_type.clone()));
            }
            if types[..index].contains(mime_type) {
                return Err(NotMimeTypes::Twice(mime_type.clone()));
            }
        }
        Ok(MimeTypes(types))
    }

    pub fn types(&self) -> &[String] {
        &self.0
    }

    /// Whether `mime_type` is one of the list's, compared without regard to case.
    pub fn contains(&self, mime_type: &str) -> bool {
        self.0.iter().any(|listed| listed.eq_ignore_ascii_case(mime_type))
    }
}

/// Whether `text` is a type and a subtype, each a registered name of RFC 6838 in lower case:
/// a letter or digit, then up to 126 more of letters, digits and `!#$&-^_.+`.
fn is_mime_type(text: &str) -> bool {
    let name = |part: &str| {
        let mut characters = part.chars();
        characters.next().is_some_and(|first| first.is_ascii_lowercase() || first.is_ascii_digit())
            && part.len() <= 127
            && characters.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "!#$&-^_.+".contains(c))
    };
    text.split_once('/').is_some_and(|(type_name, subtype)| name(type_name) && name(subtype))
}

/// Why a list is no `MimeTypes`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotMimeTypes {
    /// It names no type.
    Empty,
    /// This is not a type and a subtype in lower case without parameters.
    NotAType(String),
    /// This is listed twice.
    Twice(String),
}

impl fmt::Display for NotMimeTypes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotMimeTypes::Empty => write!(f, "no MIME type is listed"),
            NotMimeTypes::NotAType(text) => write!(f, "{text:?} is no MIME type in lower case without parameters"),
            NotMimeTypes::Twice(mime_type) => write!(f, "{mime_type} is listed twice"),
        }
    }
}

impl std::error::Error for NotMimeTypes {}

impl TryFrom<Vec<String>> for MimeTypes {
    type Error = NotMimeTypes;

    fn try_from(types: Vec<String>) -> Result<Self, NotMimeTypes> {
        MimeTypes::new(types)
    }
}

impl From<MimeTypes> for Vec<String> {
    fn from(types: MimeTypes) -> Self {
        types.0
    }
}

impl Property for MimeTypes {
    const VALUE_TYPE: &'static str = "json";

    fn into_value(&self) -> Result<Option<Value>, PropertyError> {
        Value::json(&self.0).map(Some).map_err(|error| PropertyError::SerializeError(Box::new(error)))
    }

    fn from_value(value: Option<Value>) -> Result<Self, PropertyError> {
        match value {
            Some(json @ Value::Json(_)) => {
                let types: Vec<String> = json.parse_as_json().map_err(|error| PropertyError::DeserializeError(Box::new(error)))?;
                MimeTypes::new(types)
                    .map_err(|refused| PropertyError::InvalidValue { value: refused.to_string(), ty: "MimeTypes".to_owned() })
            }
            Some(other) => Err(PropertyError::InvalidVariant { given: other, ty: "MimeTypes".to_owned() }),
            None => Err(PropertyError::Missing),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn types(list: &[&str]) -> Vec<String> {
        list.iter().map(|&mime_type| mime_type.to_owned()).collect()
    }

    // What CUPS lists for a queue with a driver, raw bytes among them.
    #[test]
    fn a_print_systems_list_of_types_is_kept_in_its_order() {
        let listed = types(&["application/pdf", "application/vnd.cups-raw", "image/urf", "application/octet-stream"]);
        assert_eq!(MimeTypes::new(listed.clone()).unwrap().types(), listed);
    }

    #[test]
    fn a_list_that_is_empty_names_something_twice_or_holds_no_type_is_refused() {
        assert_eq!(MimeTypes::new(Vec::new()), Err(NotMimeTypes::Empty));
        assert_eq!(MimeTypes::new(types(&["application/pdf", "application/pdf"])), Err(NotMimeTypes::Twice("application/pdf".into())));
        for refused in ["pdf", "application/", "/pdf", "Application/PDF", "text/plain; charset=utf-8", "application/pd f", "-x/pdf"] {
            assert_eq!(MimeTypes::new(types(&[refused])), Err(NotMimeTypes::NotAType(refused.into())), "{refused}");
        }
    }

    #[test]
    fn a_type_is_found_in_the_list_without_regard_to_case() {
        let listed = MimeTypes::new(types(&["application/pdf"])).unwrap();
        assert!(listed.contains("Application/PDF"));
        assert!(!listed.contains("application/postscript"));
    }

    // A list read back from JSON obeys the same rules as one made in code.
    #[test]
    fn json_with_a_list_that_breaks_a_rule_does_not_read() {
        assert!(serde_json::from_str::<MimeTypes>(r#"["application/pdf"]"#).is_ok());
        assert!(serde_json::from_str::<MimeTypes>("[]").is_err());
        assert!(serde_json::from_str::<MimeTypes>(r#"["PDF"]"#).is_err());
    }
}
