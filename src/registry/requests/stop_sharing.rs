use ankurah::Model;

use crate::codec::Minted;

/// Tells a registry that the station no longer shares its devices with an application: its person
/// removed the application, or the application's key retired itself. It requires
/// `ForApplication`. The registry then no longer holds the station registered for the
/// application, and sends it none of the application's jobs.
///
/// It is answered with `Done`, also when the registry no longer held the station registered, so
/// that a station that repeats the notice after losing the answer is done. A station sends it
/// straight away when it holds a connection to the registry, and otherwise on its next
/// connection, whose hello still lists the application.
///
/// A sender lists it as a component a receiver must understand, so that a receiver that does not
/// know it refuses it by name.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "registry.stop_sharing")]
#[model(id = "81jYVnI-ZXjyEfrAfIDJvukVE0rE5HuMqm1bZr1flLY")]
#[model(no_ffi)]
pub struct StopSharing {}
