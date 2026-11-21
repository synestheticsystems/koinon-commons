//! Where, for what and until when an authority holds. Each is a component of its own, which a
//! sender always lists as one a receiver must understand, so that a receiver that does not know a
//! restriction refuses the authority rather than honouring it beyond what its signer allowed.

mod allows_requests;
mod at_devices;
mod at_stations;
mod device_ids;
mod expiry;

pub use allows_requests::*;
pub use at_devices::*;
pub use at_stations::*;
pub use device_ids::*;
pub use expiry::*;
