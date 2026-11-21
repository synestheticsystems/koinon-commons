use ankurah::Model;

use crate::{codec::Minted, public_key::PublicKey};

/// The application that a request, or a record, is for, by its key: the key that signs for it,
/// which names the application the same on every node, where each node's own record of the
/// application has an entity id of its own. A registration carries it, as do a station's notice
/// that it no longer shares its devices with an application and a registry's voiding of an
/// application's slips; on a node's records it names the application of a station's job, or of a
/// registry's grant, API key or gateway job.
///
/// A registration lists it as a component a receiver must understand, so that a receiver that
/// does not know it never takes a registration for one application as one for all.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "authority.for_application")]
#[model(id = "X8qiX3Of_rHKHePD80zFJUyaei_IW3bl6efg2VnrMic")]
#[model(no_ffi)]
pub struct ForApplication {
    #[active_type(LWW)]
    #[property(id = "ejd2oVD8Pqzogtk4ZIPVmhCq5sr9V6Cj5Kytfhsm97A")]
    pub key: PublicKey,
}
