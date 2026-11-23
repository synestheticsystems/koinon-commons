use ankurah::Model;
use serde::{Deserialize, Serialize};

use crate::{codec::Minted, public_key::PublicKey};

/// That the node with this key may act on this registry. It is the whole of a node's permission
/// here: a node proves its key at every contact, and the registry looks for a grant for that key
/// that is not closed. A message names it by its id, in a `Registration` or a `ForGrant`, and
/// never carries it.
///
/// The registry writes it when a person signs the node in, whom the Ankurah Commons' `Created`
/// names. Revoking it, or signing the node out, is `Closed`. A station's devices are registered
/// on this registry while its grant is open, whatever records of them the registry still holds.
///
/// A station's grant is for one application, which its `ForApplication` names: the one the person
/// said yes to sharing the station's devices with. A key has at most one open grant for each
/// application. The registry writes a new grant for each registration and never opens a closed one
/// again, and the station's notice that it no longer shares its devices with the application names
/// the grant it ends, so that a notice that arrives late ends no grant written since. A grant for a
/// station of Pylon 0.1.0 names no application, and a key has at most one such grant open.
#[derive(Model, Minted, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[model(label = "registry.node_grant")]
#[model(id = "UTFlProeptdC4-0N_-yVsYn1P8_59Vzy9ZOZepPfpx8")]
#[model(no_ffi)]
pub struct NodeGrant {
    /// The node's public key, which never changes. A station's is the one its `Station` record
    /// holds.
    #[active_type(LWW)]
    #[property(id = "KMIVcfbV9k6tao_dsL5wACslvmTa9bd5ooV5aFVgPHA")]
    pub public_key: PublicKey,
}
