use ankurah::{envelope::EnvelopeComponent, property::Ref, Model};
use ankurah_commons::property::Timestamp;

use super::NodeGrant;

/// One registration of the station for an application, by the registry's grant to the station for
/// it: the registry's word that it keeps the station registered for the application its
/// `ForApplication` names, under this grant, since this moment. A registry writes a new grant for
/// each registration and never opens a closed one again, so the grant tells apart registrations
/// made for one application, as before and after a person removed the application and added it
/// again. The answer to a `DoRegister` carries it, the answer to a `DoListRegistrations` carries a
/// row of it for each registration, and the station keeps it on its record of the application, so
/// that its `DoEndRegistration`, sent once it no longer shares with the application, names the
/// grant, in a `ForGrant`, and ends that registration alone: a request that arrives late leaves a
/// registration made since as it was.
///
/// A receiver must understand it wherever it travels, since one that dropped it would take one
/// registration of an application for another.
#[derive(Model, EnvelopeComponent, Debug, Clone, PartialEq)]
#[model(label = "registry.registration")]
#[model(id = "kxu3xhNHkuqUQ214GmhgE6fwpStsDJHS_bg5bOzBQTU")]
#[model(no_ffi)]
#[envelope(must_understand)]
pub struct Registration {
    /// The registry's grant to the station for the application.
    #[property(id = "pJJCPTWApyE84zufQ1K9kCNQnuPrmtUXYRI9ubPqGuc")]
    #[envelope(number = 0)]
    pub grant: Ref<NodeGrant>,
    /// When the registry wrote the grant, by its own clock: the moment from which the station
    /// honours the application's permission slips, which the registry issues by the same clock, so
    /// that the two compare exactly and no slip issued before the person's yes passes. The registry
    /// gives it later than every slip it issued before, as `PermissionSlip` sets out, so that a
    /// slip issued in the same millisecond, before the grant, is still earlier.
    #[property(id = "v7sLDiY7ONT42rgCedsaJxNYdD8QkCqrTkMr52yHdLo")]
    #[envelope(number = 1)]
    pub registered: Timestamp,
}
