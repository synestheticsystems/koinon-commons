use std::fmt;

use ankurah::{property::PropertyError, property::Ref, value::Value, Property};

use crate::devices::Device;

/// Devices, each by its `Device` record, at least one and none twice, such as the devices a slip
/// narrows to. Stored as a JSON array of the records' ids, in the base64 text Ankurah writes an
/// entity's id in, in the order given.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceIds(Vec<Ref<Device>>);

impl DeviceIds {
    pub fn new(devices: Vec<Ref<Device>>) -> Result<Self, NotDeviceIds> {
        if devices.is_empty() {
            return Err(NotDeviceIds::Empty);
        }
        if let Some((index, _)) = devices.iter().enumerate().find(|(index, device)| devices[..*index].contains(device)) {
            return Err(NotDeviceIds::Twice(devices[index].to_string()));
        }
        Ok(DeviceIds(devices))
    }

    pub fn devices(&self) -> &[Ref<Device>] {
        &self.0
    }

    pub fn contains(&self, device: &Ref<Device>) -> bool {
        self.0.contains(device)
    }
}

/// Why a list of devices is no [`DeviceIds`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotDeviceIds {
    #[error("no device is listed")]
    Empty,
    /// The device, by its id's text, is listed twice.
    #[error("the device {0} is listed twice")]
    Twice(String),
}

impl Property for DeviceIds {
    const VALUE_TYPE: &'static str = "json";

    fn into_value(&self) -> Result<Option<Value>, PropertyError> {
        Ok(Some(Value::Json(self.0.iter().map(|device| serde_json::Value::String(device.id().to_base64())).collect())))
    }

    fn from_value(value: Option<Value>) -> Result<Self, PropertyError> {
        match value {
            Some(json @ Value::Json(_)) => {
                let texts: Vec<String> = json.parse_as_json().map_err(|error| PropertyError::DeserializeError(Box::new(error)))?;
                let invalid =
                    |refused: &dyn fmt::Display| PropertyError::InvalidValue { value: refused.to_string(), ty: "DeviceIds".to_owned() };
                let devices = texts.iter().map(|text| Ref::from_base64(text).map_err(|_| invalid(text))).collect::<Result<_, _>>()?;
                DeviceIds::new(devices).map_err(|refused| invalid(&refused))
            }
            Some(other) => Err(PropertyError::InvalidVariant { given: other, ty: "DeviceIds".to_owned() }),
            None => Err(PropertyError::Missing),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ankurah::EntityId;

    fn device(byte: u8) -> Ref<Device> {
        Ref::new(EntityId::from_bytes([byte; 32]))
    }

    #[test]
    fn devices_are_stored_as_a_json_array_of_their_ids() {
        let devices = DeviceIds::new(vec![device(2), device(1)]).unwrap();
        let stored = devices.into_value().unwrap();
        assert_eq!(stored, Some(Value::Json(serde_json::json!([device(2).id().to_base64(), device(1).id().to_base64()]))));
        assert_eq!(DeviceIds::from_value(stored).unwrap(), devices);
    }

    #[test]
    fn a_list_that_is_empty_names_a_device_twice_or_holds_no_id_does_not_read() {
        assert_eq!(DeviceIds::new(Vec::new()), Err(NotDeviceIds::Empty));
        assert!(matches!(DeviceIds::new(vec![device(1), device(1)]), Err(NotDeviceIds::Twice(_))));
        let twice = device(1).id().to_base64();
        for refused in [serde_json::json!([]), serde_json::json!([twice, twice]), serde_json::json!(["Pack bench 3 labels"])] {
            let read = DeviceIds::from_value(Some(Value::Json(refused.clone())));
            assert!(matches!(read, Err(PropertyError::InvalidValue { .. })), "{refused}: {read:?}");
        }
    }
}
