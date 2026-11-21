//! Components named by their models' minted ids, as one component names others: the kinds of
//! request a permission slip allows, or the components a receiver did not understand.

use std::fmt;

use ankurah::{property::PropertyError, value::Value, EntityId, Property};

use crate::codec::MintedId;

/// Components, each named by its model's minted id, at least one and none twice, such as the
/// kinds of request a permission slip allows, each by its request component's id. Stored as a
/// JSON array of the ids' texts, as `#[model(id = ...)]` writes them, in the order given.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentIds(Vec<MintedId>);

impl ComponentIds {
    pub fn new(ids: Vec<MintedId>) -> Result<Self, NotComponentIds> {
        if ids.is_empty() {
            return Err(NotComponentIds::Empty);
        }
        if let Some((index, _)) = ids.iter().enumerate().find(|(index, id)| ids[..*index].contains(id)) {
            return Err(NotComponentIds::Twice(ids[index]));
        }
        Ok(ComponentIds(ids))
    }

    pub fn ids(&self) -> &[MintedId] {
        &self.0
    }

    pub fn contains(&self, id: MintedId) -> bool {
        self.0.contains(&id)
    }
}

/// Why a list of ids is no [`ComponentIds`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotComponentIds {
    #[error("no component is listed")]
    Empty,
    #[error("{0} is listed twice")]
    Twice(MintedId),
}

impl Property for ComponentIds {
    const VALUE_TYPE: &'static str = "json";

    fn into_value(&self) -> Result<Option<Value>, PropertyError> {
        Ok(Some(Value::Json(self.0.iter().map(|id| serde_json::Value::String(id.to_string())).collect())))
    }

    fn from_value(value: Option<Value>) -> Result<Self, PropertyError> {
        match value {
            Some(json @ Value::Json(_)) => {
                let texts: Vec<String> = json.parse_as_json().map_err(|error| PropertyError::DeserializeError(Box::new(error)))?;
                let invalid =
                    |refused: &dyn fmt::Display| PropertyError::InvalidValue { value: refused.to_string(), ty: "ComponentIds".to_owned() };
                let ids = texts
                    .iter()
                    .map(|text| EntityId::from_base64(text).map(|id| MintedId::from_bytes(id.to_bytes())).map_err(|_| invalid(text)))
                    .collect::<Result<_, _>>()?;
                ComponentIds::new(ids).map_err(|refused| invalid(&refused))
            }
            Some(other) => Err(PropertyError::InvalidVariant { given: other, ty: "ComponentIds".to_owned() }),
            None => Err(PropertyError::Missing),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(byte: u8) -> MintedId {
        MintedId::from_bytes([byte; 32])
    }

    #[test]
    fn ids_are_stored_as_a_json_array_of_the_texts_the_source_declares_them_by() {
        let ids = ComponentIds::new(vec![id(2), id(1)]).unwrap();
        let stored = ids.into_value().unwrap();
        assert_eq!(stored, Some(Value::Json(serde_json::json!([id(2).to_string(), id(1).to_string()]))));
        assert_eq!(ComponentIds::from_value(stored).unwrap(), ids);
        assert!(ids.contains(id(1)) && !ids.contains(id(3)));
    }

    #[test]
    fn a_list_that_is_empty_names_an_id_twice_or_holds_no_id_does_not_read() {
        assert_eq!(ComponentIds::new(Vec::new()), Err(NotComponentIds::Empty));
        assert_eq!(ComponentIds::new(vec![id(1), id(1)]), Err(NotComponentIds::Twice(id(1))));
        for refused in [serde_json::json!([]), serde_json::json!([id(1).to_string(), id(1).to_string()]), serde_json::json!(["Print"])] {
            let read = ComponentIds::from_value(Some(Value::Json(refused.clone())));
            assert!(matches!(read, Err(PropertyError::InvalidValue { .. })), "{refused}: {read:?}");
        }
    }
}
