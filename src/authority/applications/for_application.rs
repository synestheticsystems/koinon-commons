use ankurah::Model;

use crate::{codec::Minted, public_key::PublicKey};

/// The application that a request, or a record, is for, by its key: the key that signs for it,
/// which names the application the same on every node, where each node's own record of the
/// application has an entity id of its own. A registration and the registry's answer to it carry
/// it, as do the registrations a registry's hello lists, a station's notice that it no longer
/// shares its devices with an application, and a registry's voiding of an application's slips; on
/// a node's records it names the application of a station's job or party, or of a registry's
/// grant, API key or gateway job.
///
/// A registration lists it as a component a receiver must understand, so that a receiver that
/// does not know it never takes a registration for one application as one for all.
// OPEN: a node's records name an application by its key too, with this component, rather than by
// a reference to the node's own record of the application, so that a record keeps the component
// as a message carried it. Lean: yes, since a message names an application by its key, the same
// on every node, and no record names one by a reference yet.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "authority.for_application")]
#[model(id = "X8qiX3Of_rHKHePD80zFJUyaei_IW3bl6efg2VnrMic")]
#[model(no_ffi)]
pub struct ForApplication {
    #[active_type(LWW)]
    #[property(id = "ejd2oVD8Pqzogtk4ZIPVmhCq5sr9V6Cj5Kytfhsm97A")]
    pub key: PublicKey,
}
