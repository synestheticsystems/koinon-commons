use ankurah::Model;
use serde::{Deserialize, Serialize};

/// That this node is signed in to another node, as a station is to a server. The node that
/// signs in makes the record and sends it to no one. Signing out is the Ankurah Commons'
/// `Closed`, and signing in again to the same node, the one with the same key, reopens the
/// record, so that whatever names the record, such as the origin of a job, still names that
/// node.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "nodes.node_link")]
#[model(id = "UzCe1eyjgFf9gjAV6EYbO5DoKsLvCvb5JEAPQP-dPxg")]
#[model(no_ffi)]
pub struct NodeLink {
    /// The other node's public key, which never changes. At every contact this node checks
    /// that the node it reached holds it, and refuses a node that answers at the address
    /// under another key.
    // OPEN: its text form follows Ankurah's node keys, as the station's does.
    #[active_type(LWW)]
    pub public_key: String,
    /// Where the other node is reached, as the person who signed in gave it. It may change,
    /// as when a server moves.
    // OPEN: its form follows how a node signs in, such as the address of a server's page.
    #[active_type(LWW)]
    pub address: String,
    /// What a person calls the other node.
    #[active_type(LWW)]
    pub name: String,
}
