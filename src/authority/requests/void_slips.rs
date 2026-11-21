use ankurah::Model;

use crate::codec::Minted;

/// Tells a station that the permission slips of an application it is registered for, issued
/// before a moment, are void. It requires `SlipsVoided` and `ForApplication`, the application, and
/// comes from a registry alone: a station takes it only down the connection of the registry the
/// application came through. It is answered with `Done`.
///
/// A sender lists it as a component a receiver must understand, so that a receiver that does not
/// know it refuses it by name.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "authority.void_slips")]
#[model(id = "U2YMdH2cGkOHx_1F1lGjGImbJeLC4nF5kVrX1c-jBFE")]
#[model(no_ffi)]
pub struct VoidSlips {}
