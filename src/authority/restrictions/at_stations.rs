use ankurah::Model;

use crate::{codec::Minted, public_key::PublicKeys};

/// The stations at which an authority is honoured, by their keys. On a permission slip they are
/// where its party may dial, with no registry to ask, and where the slip is honoured; on a signed
/// request, the one station the request is for, so that a copy of it is honoured at no other. A
/// station honours an authority only where this names it. Every slip and every signed request has
/// it.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "authority.at_stations")]
#[model(id = "y02GCiLKsbMwV6S0gQnBItGRkGWm5rpsKbS4AI8Mo_w")]
#[model(no_ffi)]
pub struct AtStations {
    #[active_type(LWW)]
    #[property(id = "vgYEyJdO3UnhZho8U5I1c_7sddXZEEDB1v9vbEJhFxY")]
    pub stations: PublicKeys,
}
