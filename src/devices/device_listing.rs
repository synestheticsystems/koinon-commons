use ankurah::{
    envelope::{EntityRows, EnvelopeComponent},
    Model,
};

/// The devices a station lists, and the station itself: the answer to a `DoListDevices`, and to a
/// computer's `InvitationClaim`. A row for each device, under the device's entity id, holds
/// `Device` with `Printer`, and with `Media` once someone has said what the printer is loaded
/// with, or `Device` with `Scale`; a row under the station's own entity id holds its `Station`,
/// which each `Device` names. Where the operation answered requires the station's row, its
/// documentation says so.
#[derive(Model, EnvelopeComponent, Debug, Clone, PartialEq)]
#[model(label = "devices.device_listing")]
#[model(id = "5dQX5_KdTd7tVR5xmZFtVqCEjAkD1pX1KJvxd6jFzMw")]
#[model(no_ffi)]
pub struct DeviceListing {
    /// A row for each device listed, and one for the station, in the order the station gives.
    #[active_type(LWW)]
    #[property(id = "ChOR5oF6hL2ASAkDwLQ880qatEooaR4MC_uL4iGUtm0")]
    #[envelope(number = 0)]
    pub entries: EntityRows,
}
