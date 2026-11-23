//! MIME types, by which a document says what format it is in and a printer says which formats
//! its print system takes.

use std::fmt;

use ankurah::{property::PropertyError, value::Value, Property};
use serde::{Deserialize, Serialize};

use crate::stored_forms::first_repeat;

/// A MIME type: a type and a subtype without parameters, in lower case, of the characters a
/// registered name may hold (RFC 6838), such as `application/pdf`. MIME types compare without
/// regard to case, so a `MimeType` keeps one spelling of each. Stored as its text.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct MimeType(String);

impl MimeType {
    pub fn new(text: impl Into<String>) -> Result<Self, NotMimeType> {
        let text = text.into();
        if is_mime_type(&text) {
            Ok(MimeType(text))
        } else {
            Err(NotMimeType(text))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
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

impl fmt::Display for MimeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Why a text is no `MimeType`: it is not a type and a subtype in lower case without
/// parameters. It holds the text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotMimeType(pub String);

impl fmt::Display for NotMimeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} is no MIME type in lower case without parameters", self.0)
    }
}

impl std::error::Error for NotMimeType {}

impl TryFrom<String> for MimeType {
    type Error = NotMimeType;

    fn try_from(text: String) -> Result<Self, NotMimeType> {
        MimeType::new(text)
    }
}

impl From<MimeType> for String {
    fn from(mime_type: MimeType) -> Self {
        mime_type.0
    }
}

impl Property for MimeType {
    const VALUE_TYPE: &'static str = "string";

    fn into_value(&self) -> Result<Option<Value>, PropertyError> {
        Ok(Some(Value::String(self.0.clone())))
    }

    fn from_value(value: Option<Value>) -> Result<Self, PropertyError> {
        match value {
            Some(Value::String(text)) => {
                MimeType::new(text).map_err(|refused| PropertyError::InvalidValue { value: refused.0, ty: "MimeType".to_owned() })
            }
            Some(other) => Err(PropertyError::InvalidVariant { given: other, ty: "MimeType".to_owned() }),
            None => Err(PropertyError::Missing),
        }
    }
}

/// A list of MIME types, at least one and none twice, such as a print system lists for a
/// printer. Stored as a JSON array of the types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Vec<MimeType>", into = "Vec<MimeType>")]
pub struct MimeTypes(Vec<MimeType>);

impl MimeTypes {
    pub fn new(types: Vec<MimeType>) -> Result<Self, NotMimeTypes> {
        if types.is_empty() {
            return Err(NotMimeTypes::Empty);
        }
        if let Some(repeated) = first_repeat(&types) {
            return Err(NotMimeTypes::Twice(repeated.clone()));
        }
        Ok(MimeTypes(types))
    }

    pub fn types(&self) -> &[MimeType] {
        &self.0
    }

    /// Whether `mime_type` is one of the list's, compared without regard to case.
    pub fn contains(&self, mime_type: &str) -> bool {
        self.0.iter().any(|listed| listed.as_str().eq_ignore_ascii_case(mime_type))
    }
}

/// Why a list of MIME types is no `MimeTypes`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotMimeTypes {
    /// It names no type.
    Empty,
    /// This is listed twice.
    Twice(MimeType),
}

impl fmt::Display for NotMimeTypes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotMimeTypes::Empty => write!(f, "no MIME type is listed"),
            NotMimeTypes::Twice(mime_type) => write!(f, "{mime_type} is listed twice"),
        }
    }
}

impl std::error::Error for NotMimeTypes {}

impl TryFrom<Vec<MimeType>> for MimeTypes {
    type Error = NotMimeTypes;

    fn try_from(types: Vec<MimeType>) -> Result<Self, NotMimeTypes> {
        MimeTypes::new(types)
    }
}

impl From<MimeTypes> for Vec<MimeType> {
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
                let texts: Vec<String> = json.parse_as_json().map_err(|error| PropertyError::DeserializeError(Box::new(error)))?;
                // A type of another shape, like a list that breaks the list's rules, is a value
                // the list refuses, not JSON that does not parse.
                let invalid =
                    |refused: &dyn fmt::Display| PropertyError::InvalidValue { value: refused.to_string(), ty: "MimeTypes".to_owned() };
                let types = texts.into_iter().map(MimeType::new).collect::<Result<_, _>>().map_err(|refused| invalid(&refused))?;
                MimeTypes::new(types).map_err(|refused| invalid(&refused))
            }
            Some(other) => Err(PropertyError::InvalidVariant { given: other, ty: "MimeTypes".to_owned() }),
            None => Err(PropertyError::Missing),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mime_type(text: &str) -> MimeType {
        MimeType::new(text).unwrap()
    }

    fn types(list: &[&str]) -> Vec<MimeType> {
        list.iter().map(|&text| mime_type(text)).collect()
    }

    #[test]
    fn a_type_and_a_subtype_in_lower_case_read_as_a_mime_type() {
        for text in ["application/pdf", "application/vnd.cups-raw", "image/urf", "application/vnd.hp-pcl", "model/3mf"] {
            let read = mime_type(text);
            assert_eq!((read.as_str(), read.to_string()), (text, text.to_owned()));
        }
    }

    #[test]
    fn text_of_any_other_shape_is_no_mime_type() {
        for refused in [
            "pdf",
            "application/",
            "/pdf",
            "Application/PDF",
            "text/plain; charset=utf-8",
            "application/pd f",
            "-x/pdf",
            "application/pdf/a",
        ] {
            assert_eq!(MimeType::new(refused), Err(NotMimeType(refused.to_owned())), "{refused}");
        }
    }

    // In JSON, such as a print job's document carries, a MIME type is a string, and a string of
    // another shape does not read.
    #[test]
    fn a_mime_type_is_a_json_string_and_one_of_another_shape_does_not_read() {
        let json = serde_json::to_string(&mime_type("application/pdf")).unwrap();
        assert_eq!(json, r#""application/pdf""#);
        assert_eq!(serde_json::from_str::<MimeType>(&json).unwrap(), mime_type("application/pdf"));
        assert!(serde_json::from_str::<MimeType>(r#""Application/PDF""#).is_err());
    }

    // A node stores a MIME type as its text, and does not read back a stored text of another
    // shape.
    #[test]
    fn a_mime_type_is_stored_as_its_text() {
        let stored = mime_type("application/pdf").into_value().unwrap();
        assert_eq!(stored, Some(Value::String("application/pdf".to_owned())));
        assert_eq!(MimeType::from_value(stored).unwrap(), mime_type("application/pdf"));
        let refused = MimeType::from_value(Some(Value::String("PDF".to_owned())));
        assert!(matches!(&refused, Err(PropertyError::InvalidValue { value, .. }) if value == "PDF"), "{refused:?}");
    }

    // What CUPS lists for a queue with a driver, raw bytes among them.
    #[test]
    fn a_print_systems_list_of_types_is_kept_in_its_order() {
        let listed = types(&["application/pdf", "application/vnd.cups-raw", "image/urf", "application/octet-stream"]);
        assert_eq!(MimeTypes::new(listed.clone()).unwrap().types(), listed);
    }

    #[test]
    fn a_list_that_is_empty_or_names_a_type_twice_is_refused() {
        assert_eq!(MimeTypes::new(Vec::new()), Err(NotMimeTypes::Empty));
        assert_eq!(MimeTypes::new(types(&["application/pdf", "application/pdf"])), Err(NotMimeTypes::Twice(mime_type("application/pdf"))));
    }

    #[test]
    fn a_type_is_found_in_the_list_without_regard_to_case() {
        let listed = MimeTypes::new(types(&["application/pdf"])).unwrap();
        assert!(listed.contains("Application/PDF"));
        assert!(!listed.contains("application/postscript"));
    }

    // A list is a JSON array of its types' text, and one read back from JSON obeys the same rules
    // as one made in code: empty, a type named twice and a type of another shape do not read.
    #[test]
    fn json_with_a_list_that_breaks_a_rule_does_not_read() {
        let listed = MimeTypes::new(types(&["application/pdf", "application/vnd.cups-raw"])).unwrap();
        let json = serde_json::to_string(&listed).unwrap();
        assert_eq!(json, r#"["application/pdf","application/vnd.cups-raw"]"#);
        assert_eq!(serde_json::from_str::<MimeTypes>(&json).unwrap(), listed);
        for refused in ["[]", r#"["application/pdf", "application/pdf"]"#, r#"["PDF"]"#] {
            assert!(serde_json::from_str::<MimeTypes>(refused).is_err(), "{refused}");
        }
    }

    // As stored on a node, a list that breaks a rule, or holds a type of another shape, is a
    // value the list refuses.
    #[test]
    fn a_stored_list_that_breaks_a_rule_does_not_read() {
        let listed = MimeTypes::new(types(&["application/pdf"])).unwrap();
        assert_eq!(MimeTypes::from_value(listed.into_value().unwrap()).unwrap(), listed);
        for refused in [r#"[]"#, r#"["application/pdf", "application/pdf"]"#, r#"["PDF"]"#] {
            let stored = Value::json(&serde_json::from_str::<serde_json::Value>(refused).unwrap()).unwrap();
            let read = MimeTypes::from_value(Some(stored));
            assert!(matches!(read, Err(PropertyError::InvalidValue { .. })), "{refused}: {read:?}");
        }
    }
}
