use ankurah::{envelope::EnvelopeComponent, Model};

/// Asks a registry which registrations it keeps for the station, which it knows by the key the
/// connection proves, with the voidings of slips the station may have missed while it was not
/// connected, so that a station that has just connected sets itself and its registry straight.
/// Beside `Request`, it requires nothing.
///
/// It is answered with `Response` and `KeptRegistrations`, a row for each registration the
/// registry keeps for the station, with the latest voiding of its application's slips. A station
/// sends a `DoEndRegistration` for each registration whose grant its record of a removed or
/// retired application names, and applies each voiding as it applies the same news on any other
/// path. An answer with no row says that the registry keeps no registration for the station.
///
/// A receiver must understand it, so that one that does not know it refuses the request, naming
/// it.
// OPEN: how a station learns of a retirement it missed while it was not connected, which this
// answer no longer carries. Either a component of the answer, named for what it holds, holds the
// bytes of each such retirement's signed envelope, or the registry sends each as its own request
// down the connection once it has answered.
#[derive(Model, EnvelopeComponent, Debug, Clone, PartialEq)]
#[model(label = "registry.do_list_registrations")]
#[model(id = "IbuVq2hjcL72dFRywyLp-0uf98sBoiv2boj9jRoihEc")]
#[model(no_ffi)]
#[envelope(must_understand)]
pub struct DoListRegistrations {}
