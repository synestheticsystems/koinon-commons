use ankurah::Model;

use crate::codec::Minted;

/// Asks a registry which applications it holds the station registered for, which it knows by the
/// key the connection proves, with the news the station may have missed while it was not
/// connected, so that a station that has just connected sets itself and its registry straight.
///
/// It is answered with `RegisteredFor`, and a record, under no entity id, for each piece of news:
/// a `SignedRetirement` for each application the registry hosted that retired its key and that
/// the station is still registered for, and `SlipsVoided` with its `ForApplication` for each
/// application whose slips the registry has voided. The station applies each as it applies the
/// same news on any other path.
///
/// A sender lists it as a component a receiver must understand, so that a receiver that does not
/// know it refuses it by name.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "registry.hello")]
#[model(id = "__sX5oDeg9uelWRCtS6RBwwzCdFJ-vS0bxgQDjs9_3Q")]
#[model(no_ffi)]
pub struct Hello {}
