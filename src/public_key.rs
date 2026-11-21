//! Public keys, by which components name stations, parties and applications: where an authority
//! holds, whom a permission slip is for, which application a request is for, and whose key
//! signed an envelope that another carries.

use std::fmt;

use ankurah::{property::PropertyError, value::Value, Property};
use ed25519_dalek::VerifyingKey;

use crate::hex::{from_lower_hex, lower_hex};

/// The public half of an Ed25519 key pair: a station's or a party's, the key of its iroh
/// endpoint, which its connection proves; or an application's, whose secret half signs for it.
/// Stored and carried as its text, its 32 bytes as 64 lower-case hexadecimal digits, as iroh
/// writes a key, so that a key has one text and two texts name the same key exactly when they
/// are equal. A text of any other shape is no key, and neither are 32 bytes that name no point of
/// the curve, since no key pair has them.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PublicKey([u8; 32]);

impl PublicKey {
    pub fn from_bytes(bytes: [u8; 32]) -> Result<Self, NotAPublicKey> {
        match VerifyingKey::from_bytes(&bytes) {
            Ok(_) => Ok(PublicKey(bytes)),
            Err(_) => Err(NotAPublicKey(lower_hex(&bytes))),
        }
    }

    /// Reads the text [`PublicKey::text`] writes, and no other.
    pub fn from_text(text: &str) -> Result<Self, NotAPublicKey> {
        let refused = || NotAPublicKey(text.to_owned());
        PublicKey::from_bytes(from_lower_hex(text).ok_or_else(refused)?).map_err(|_| refused())
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// The key's text: its 32 bytes as 64 lower-case hexadecimal digits.
    pub fn text(&self) -> String {
        lower_hex(&self.0)
    }

    /// The key, as a signature is checked with it.
    pub(crate) fn verifying_key(&self) -> VerifyingKey {
        VerifyingKey::from_bytes(&self.0).expect("a PublicKey names a point of the curve, which was checked when it was made")
    }
}

impl From<VerifyingKey> for PublicKey {
    fn from(key: VerifyingKey) -> Self {
        PublicKey(key.to_bytes())
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text())
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("PublicKey").field(&self.text()).finish()
    }
}

/// Why a text, or 32 bytes, are no [`PublicKey`]: not 64 lower-case hexadecimal digits, or no
/// point of the curve. It holds the text, or the bytes' text.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0:?} is no Ed25519 public key in lower-case hexadecimal")]
pub struct NotAPublicKey(pub String);

impl Property for PublicKey {
    const VALUE_TYPE: &'static str = "string";

    fn into_value(&self) -> Result<Option<Value>, PropertyError> {
        Ok(Some(Value::String(self.text())))
    }

    fn from_value(value: Option<Value>) -> Result<Self, PropertyError> {
        match value {
            Some(Value::String(text)) => {
                PublicKey::from_text(&text).map_err(|refused| PropertyError::InvalidValue { value: refused.0, ty: "PublicKey".to_owned() })
            }
            Some(other) => Err(PropertyError::InvalidVariant { given: other, ty: "PublicKey".to_owned() }),
            None => Err(PropertyError::Missing),
        }
    }
}

/// Public keys, at least one and none twice, such as the stations at which an authority holds.
/// Stored as a JSON array of the keys' texts, in the order given.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicKeys(Vec<PublicKey>);

impl PublicKeys {
    pub fn new(keys: Vec<PublicKey>) -> Result<Self, NotPublicKeys> {
        if keys.is_empty() {
            return Err(NotPublicKeys::Empty);
        }
        if let Some((index, _)) = keys.iter().enumerate().find(|(index, key)| keys[..*index].contains(key)) {
            return Err(NotPublicKeys::Twice(keys[index]));
        }
        Ok(PublicKeys(keys))
    }

    pub fn keys(&self) -> &[PublicKey] {
        &self.0
    }

    pub fn contains(&self, key: &PublicKey) -> bool {
        self.0.contains(key)
    }
}

/// Why a list of keys is no [`PublicKeys`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotPublicKeys {
    #[error("no key is listed")]
    Empty,
    #[error("{0} is listed twice")]
    Twice(PublicKey),
}

impl Property for PublicKeys {
    const VALUE_TYPE: &'static str = "json";

    fn into_value(&self) -> Result<Option<Value>, PropertyError> {
        Ok(Some(Value::Json(self.0.iter().map(|key| serde_json::Value::String(key.text())).collect())))
    }

    fn from_value(value: Option<Value>) -> Result<Self, PropertyError> {
        match value {
            Some(json @ Value::Json(_)) => {
                let texts: Vec<String> = json.parse_as_json().map_err(|error| PropertyError::DeserializeError(Box::new(error)))?;
                // A key of another shape, like a list that breaks the list's rules, is a value the
                // list refuses, not JSON that does not parse.
                let invalid =
                    |refused: &dyn fmt::Display| PropertyError::InvalidValue { value: refused.to_string(), ty: "PublicKeys".to_owned() };
                let keys =
                    texts.iter().map(|text| PublicKey::from_text(text)).collect::<Result<_, _>>().map_err(|refused| invalid(&refused))?;
                PublicKeys::new(keys).map_err(|refused| invalid(&refused))
            }
            Some(other) => Err(PropertyError::InvalidVariant { given: other, ty: "PublicKeys".to_owned() }),
            None => Err(PropertyError::Missing),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;

    fn key(seed: u8) -> PublicKey {
        PublicKey::from(SigningKey::from_bytes(&[seed; 32]).verifying_key())
    }

    #[test]
    fn a_key_reads_back_from_its_text_and_as_stored() {
        let key = key(7);
        assert_eq!(key.text().len(), 64);
        assert_eq!(PublicKey::from_text(&key.text()), Ok(key));
        assert_eq!(PublicKey::from_value(key.into_value().unwrap()).unwrap(), key);
    }

    // The text of 32 bytes that make no point of the curve is 64 lower-case hexadecimal digits
    // all the same: a key's text in upper case, or a hash's, is no key.
    #[test]
    fn a_text_of_another_shape_or_bytes_that_make_no_point_are_no_key() {
        let text = key(7).text();
        let no_point = format!("02{}", "0".repeat(62));
        assert!(PublicKey::from_bytes(from_lower_hex(&no_point).unwrap()).is_err(), "the bytes {no_point} name a point");
        for refused in [text.to_uppercase(), text[..62].to_owned(), format!("{text}00"), no_point] {
            assert_eq!(PublicKey::from_text(&refused), Err(NotAPublicKey(refused.clone())));
            let read = PublicKey::from_value(Some(Value::String(refused.clone())));
            assert!(matches!(&read, Err(PropertyError::InvalidValue { value, .. }) if *value == refused), "{read:?}");
        }
    }

    #[test]
    fn a_list_of_keys_is_stored_as_a_json_array_of_their_texts_in_its_order() {
        let keys = PublicKeys::new(vec![key(2), key(1)]).unwrap();
        let stored = keys.into_value().unwrap();
        assert_eq!(stored, Some(Value::Json(serde_json::json!([key(2).text(), key(1).text()]))));
        assert_eq!(PublicKeys::from_value(stored).unwrap(), keys);
    }

    #[test]
    fn a_list_that_is_empty_names_a_key_twice_or_holds_no_key_does_not_read() {
        assert_eq!(PublicKeys::new(Vec::new()), Err(NotPublicKeys::Empty));
        assert_eq!(PublicKeys::new(vec![key(1), key(2), key(1)]), Err(NotPublicKeys::Twice(key(1))));
        for refused in [serde_json::json!([]), serde_json::json!([key(1).text(), key(1).text()]), serde_json::json!(["station"])] {
            let read = PublicKeys::from_value(Some(Value::Json(refused.clone())));
            assert!(matches!(read, Err(PropertyError::InvalidValue { .. })), "{refused}: {read:?}");
        }
    }
}
