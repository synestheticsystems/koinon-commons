use ankurah::Model;

use crate::codec::Minted;

/// Tells a registry that the station no longer shares its devices with an application under one
/// registration: its person removed the application, or the application's key retired itself. It
/// requires `ForApplication` and `ForGrant`, the grant of the `Registration` the station keeps on
/// its record of the application, as the registry's answer to its registration gave it, which a
/// sender lists as a component a receiver must understand; the notice needs nothing else of the
/// registration, such as the moment it was written, which the registry knows by the grant. The
/// registry ends that registration, closing its grant if the grant is still open and is this
/// station's for the application, and no other, so that a notice that arrives late, after the
/// person added the application again, leaves the newer registration as it was. It then sends
/// none of the application's jobs under the grant it closed.
///
/// It is answered with `Done`, also when the grant was closed already, so that a station that
/// repeats the notice after losing the answer is done. A station sends it straight away when it
/// is connected to the registry, and otherwise on its next connection, whose hello still lists
/// the registration.
///
/// A sender lists it as a component a receiver must understand, so that a receiver that does not
/// know it refuses it by name.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "registry.stop_sharing")]
#[model(id = "81jYVnI-ZXjyEfrAfIDJvukVE0rE5HuMqm1bZr1flLY")]
#[model(no_ffi)]
pub struct StopSharing {}
