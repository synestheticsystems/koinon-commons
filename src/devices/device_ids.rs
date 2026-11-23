use ankurah::{property::Ref, EntityId};

use super::Device;
use crate::stored_forms::{distinct_list, ListedText};

distinct_list! {
    /// Devices, each by its `Device` record, at least one and none twice, in the order given: the
    /// devices a permission slip narrows to, or a station's person offered a computer they paired with
    /// the station. Stored as a JSON array of the records' ids, in the base64 text Ankurah writes an
    /// entity's id in.
    pub struct DeviceIds(Vec<Ref<Device>>), each a "device", given by devices;

    /// Why a list of devices is no [`DeviceIds`].
    pub enum NotDeviceIds;
}

impl ListedText for Ref<Device> {
    fn listed_text(&self) -> String {
        self.id().to_base64()
    }

    fn from_listed_text(text: &str) -> Result<Self, String> {
        EntityId::from_base64(text).map(Ref::new).map_err(|_| format!("{text:?} is no record's id"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ankurah::{property::PropertyError, value::Value, Property};

    fn device(byte: u8) -> Ref<Device> {
        Ref::new(EntityId::from_bytes([byte; 32]))
    }

    #[test]
    fn devices_are_stored_as_a_json_array_of_their_ids_in_their_order() {
        let devices = DeviceIds::new(vec![device(2), device(1)]).unwrap();
        let stored = devices.into_value().unwrap();
        assert_eq!(stored, Some(Value::Json(serde_json::json!([device(2).id().to_base64(), device(1).id().to_base64()]))));
        assert_eq!(DeviceIds::from_value(stored).unwrap(), devices);
        assert!(devices.contains(&device(1)) && !devices.contains(&device(3)));
    }

    #[test]
    fn a_list_that_is_empty_names_a_device_twice_or_holds_no_id_does_not_read() {
        assert_eq!(DeviceIds::new(Vec::new()), Err(NotDeviceIds::Empty));
        assert_eq!(DeviceIds::new(vec![device(1), device(2), device(1)]), Err(NotDeviceIds::Twice(device(1))));
        let twice = device(1).id().to_base64();
        for refused in [serde_json::json!([]), serde_json::json!([twice, twice]), serde_json::json!(["Pack bench 3 labels"])] {
            let read = DeviceIds::from_value(Some(Value::Json(refused.clone())));
            assert!(matches!(read, Err(PropertyError::InvalidValue { .. })), "{refused}: {read:?}");
        }
    }
}
