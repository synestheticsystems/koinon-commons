use ankurah::{envelope::EnvelopeComponent, property::Ref, Model};

use super::NodeGrant;

/// The grant a request is for, by its record: how a station's `DoEndRegistration` names the one
/// registration it ends, by the grant of the `Registration` the registry answered that
/// registration with, so that a request that arrives late ends that registration alone and never
/// one written since. The request needs nothing else of the registration, since the registry knows
/// the rest by its grant.
///
/// A receiver must understand it, since one that dropped it would take the request for one that
/// ends every registration of the application.
#[derive(Model, EnvelopeComponent, Debug, Clone, PartialEq)]
#[model(label = "registry.for_grant")]
#[model(id = "hMIGomGMld_G7j1PiRg9Ev_NIsNJ8giRWp3zQ1tt7mM")]
#[model(no_ffi)]
#[envelope(must_understand)]
pub struct ForGrant {
    #[property(id = "gNg_58aNHV4midkQ9fgqp88XdE6dynuTJtJJGhpnrHs")]
    #[envelope(number = 0)]
    pub grant: Ref<NodeGrant>,
}
