use ankurah::Model;

use crate::codec::Minted;

/// Asks a station which of its devices the request's authority reaches, so that a party learns
/// the ids of the devices it may use with no registry to ask. It requires an authority: a
/// `SignedRequest` it travels inside, which reaches every device the station offers, or a
/// `PresentedSlip` beside it, which reaches the devices the slip covers, of the kinds it allows.
///
/// It is answered with a record for each device the authority reaches, under the device's entity
/// id: `Device` with `Printer`, and with `Media` once someone has said what the printer is loaded
/// with, or `Device` with `Scale`; the station may add its own `Station` record, which each
/// `Device` names. A device's `Device` travels whole, its `system_name` with it: a sender leaves
/// out a whole component and never one of its properties. A slip that covers devices none of
/// which this station offers is refused as `not_authorized`, rather than answered with no record.
///
/// A sender lists it as a component a receiver must understand, so that a receiver that does not
/// know it refuses it by name.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "devices.list_devices")]
#[model(id = "S6MDvsyTIwpzotERnFh5Bbcg5Q1DFFwM2PZFufhJh_c")]
#[model(no_ffi)]
pub struct ListDevices {}
