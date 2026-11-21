use ankurah::Model;

use crate::codec::Minted;

/// Hands a station an application's retirement, which needs no authority beside its own
/// signature, so that whoever learns that an application's key has leaked can stop a station that
/// no registry reaches, as on a network with no internet. A registry sends it down its held
/// connection to each station registered for the application, and anyone may send it on the
/// direct path. It requires `SignedRetirement`.
///
/// It is answered with `Done`, whether or not the station shares its devices with the
/// application, so that the answer tells no one which applications it shares with.
///
/// A sender lists it as a component a receiver must understand, so that a receiver that does not
/// know it refuses it by name.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "authority.retire")]
#[model(id = "V-THPjdSJ9gzsmYcnM4ixoqW6FzEjdAaMMedBdC1n0s")]
#[model(no_ffi)]
pub struct Retire {}
