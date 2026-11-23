use ankurah::{property::Ref, Model};

use super::NodeGrant;
use crate::codec::Minted;

/// The grant a request is for, by its record: how a station's notice that it no longer shares its
/// devices with an application (`StopSharing`) names the one registration it ends, by the grant of
/// the `Registration` the registry answered that registration with, so that a notice that arrives
/// late ends that registration alone and never one written since. The notice needs nothing else of
/// the registration, since the registry knows the rest by its grant.
///
/// A sender lists it as a component a receiver must understand, since a receiver that dropped it
/// would take the notice for one that ends every registration of the application.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "registry.for_grant")]
#[model(id = "hMIGomGMld_G7j1PiRg9Ev_NIsNJ8giRWp3zQ1tt7mM")]
#[model(no_ffi)]
pub struct ForGrant {
    #[property(id = "gNg_58aNHV4midkQ9fgqp88XdE6dynuTJtJJGhpnrHs")]
    pub grant: Ref<NodeGrant>,
}
