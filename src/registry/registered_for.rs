use ankurah::Model;

use crate::{codec::Minted, public_key::PublicKeys};

/// The applications a registry holds the station registered for, by their keys: those whose
/// gateway jobs it sends down the station's connection. A station that has just connected learns
/// from it which of its registrations the registry still holds, and tells the registry of each
/// application it no longer shares its devices with.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "registry.registered_for")]
#[model(id = "IWPwTgvETyhsQYKjxY03CQggF8p96uQyllwR0W8FZWs")]
#[model(no_ffi)]
pub struct RegisteredFor {
    /// None when the registry holds the station registered for no application.
    #[active_type(LWW)]
    #[property(id = "wqaIU0t5jp6MTpMxP_z08Al2R0t1W59TrU7mbY58_Rc")]
    pub applications: Option<PublicKeys>,
}
