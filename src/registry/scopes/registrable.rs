use ankurah::Model;

use crate::codec::Minted;

/// That the person who signed in may register the station in the scope or sub-scope on this
/// entity: on a record of the answer to a sign-in, beside its `Scope`. A person who is a member of
/// a scope as a whole may register the station in the scope and in each of its sub-scopes; one
/// who is a member of some sub-scopes alone, only in those, and their scope comes as `Scope` alone,
/// so that the station can show the sub-scopes under its name.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "registry.registrable")]
#[model(id = "cJZLIpgDIVedWUDZHUVnT_JkKpmdzOoZVwd8VlyuYdg")]
#[model(no_ffi)]
pub struct Registrable {}
