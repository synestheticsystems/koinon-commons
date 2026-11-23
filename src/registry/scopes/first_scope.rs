use ankurah::Model;

use crate::{codec::Minted, name::Name};

/// A first scope for the station to serve in, which the registry makes with this name and with the
/// person as its member: how a person who is a member of no scope registers a station. A
/// registration carries it in place of a `Placement`, and one that carries both is refused as
/// `bad_request`.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "registry.first_scope")]
#[model(id = "k8PbpVkHbVTYqBRC8OvuzfbzZkcriMsgtl_M8KBtH6I")]
#[model(no_ffi)]
pub struct FirstScope {
    /// What the person calls the scope, such as "Acme", under which the registry lists it.
    #[active_type(LWW)]
    #[property(id = "VDxevSz9tQOOEjYYdgiJH6qj3TgplFzaSShY270pKW0")]
    pub name: Name,
}
