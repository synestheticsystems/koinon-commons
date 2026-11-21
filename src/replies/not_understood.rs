use ankurah::Model;

use crate::{codec::Minted, component_ids::ComponentIds};

/// Names every component a refused request listed as one a receiver must understand that the side
/// asked does not understand, so that the sender learns in one answer what it may leave out, or
/// that the side it asked is too old for what it asked. It comes beside a `Refusal` whose code is
/// [`crate::replies::Refusal::NOT_UNDERSTOOD`].
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "replies.not_understood")]
#[model(id = "41_X-OqloSwZ1O0EaXbFmy7owalg4EvwnJld2tIUbT0")]
#[model(no_ffi)]
pub struct NotUnderstood {
    #[active_type(LWW)]
    #[property(id = "EmwMiA69orwdbYltfP0vGTsLNtft7AoactXdFozmkRA")]
    pub components: ComponentIds,
}
