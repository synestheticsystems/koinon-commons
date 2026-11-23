//! Components named by their models' minted ids, as one component names others: the kinds of
//! request a permission slip allows, or the components a receiver did not understand.

use ankurah::EntityId;

use crate::{
    codec::MintedId,
    stored_forms::{distinct_list, ListedText},
};

distinct_list! {
    /// Components, each named by its model's minted id, at least one and none twice, in the order
    /// given, such as the kinds of request a permission slip allows, each by its request
    /// component's id. Stored as a JSON array of the ids' texts, as `#[model(id = ...)]` writes them.
    pub struct ComponentIds(Vec<MintedId>), each a "component", given by ids;

    /// Why a list of ids is no [`ComponentIds`].
    pub enum NotComponentIds;
}

impl ListedText for MintedId {
    fn listed_text(&self) -> String {
        self.to_string()
    }

    fn from_listed_text(text: &str) -> Result<Self, String> {
        EntityId::from_base64(text).map(|id| MintedId::from_bytes(id.to_bytes())).map_err(|_| format!("{text:?} is no minted id"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ankurah::{property::PropertyError, value::Value, Property};

    fn id(byte: u8) -> MintedId {
        MintedId::from_bytes([byte; 32])
    }

    #[test]
    fn ids_are_stored_as_a_json_array_of_the_texts_the_source_declares_them_by() {
        let ids = ComponentIds::new(vec![id(2), id(1)]).unwrap();
        let stored = ids.into_value().unwrap();
        assert_eq!(stored, Some(Value::Json(serde_json::json!([id(2).to_string(), id(1).to_string()]))));
        assert_eq!(ComponentIds::from_value(stored).unwrap(), ids);
        assert!(ids.contains(&id(1)) && !ids.contains(&id(3)));
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
