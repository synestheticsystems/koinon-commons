use ankurah::{envelope::EnvelopeComponent, Model};

use crate::devices::DeviceIds;

/// Narrows a permission slip to these devices, at the stations its `AtStations` names, so that a
/// party may use some of a station's devices and not the rest. A slip without it covers every
/// device at its stations, those a station adds later among them. Each device's record was made by
/// its own station, so a device named here at a station the slip does not name is covered by
/// nothing.
// OPEN: a slip always names its stations, and may narrow them to devices, rather than covering
// either stations or devices; so that a party with a slip for devices knows which station to dial
// with no registry to ask. Lean: yes.
#[derive(Model, EnvelopeComponent, Debug, Clone, PartialEq)]
#[model(label = "authority.at_devices")]
#[model(id = "OMtujR_QuaFt7MgwwiGPAobBHnWk-phgH8iywZwP5I8")]
#[model(no_ffi)]
#[envelope(must_understand)]
pub struct AtDevices {
    #[active_type(LWW)]
    #[property(id = "r4GkW54WXmy4tyIAUH4ZQv7BP4uEWQVgf4iNyoKjT-E")]
    #[envelope(number = 0)]
    pub devices: DeviceIds,
}
