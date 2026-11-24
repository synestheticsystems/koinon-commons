use ankurah::{envelope::EnvelopeComponent, Model};

/// Asks a registry to end one registration of the station for an application, because the station
/// no longer shares its devices with the application: its person removed the application, or the
/// application's key retired itself. Beside `Request`, it requires `ForApplication` and
/// `ForGrant`, the grant of the `Registration` the station keeps on its record of the application,
/// as the registry's answer to its registration gave it; the request needs nothing else of the
/// registration, such as the moment it was written, which the registry knows by the grant. The
/// registry ends that registration, closing its grant if the grant is still open, is for that
/// application, and is for the station whose key the connection proves, and no other: so that a
/// request that arrives late, after the person added the application again, leaves the newer
/// registration as it was, and one that another station sends, under a grant whose id it learnt,
/// ends nothing. It then sends none of the application's jobs under the grant it closed.
///
/// It is answered with `Response` alone, also when the grant was closed already or was another
/// station's, so that a station that repeats the request after losing the answer is done, and the
/// answer tells no station of another's grants. A station sends it straight away when it is
/// connected to the registry, and otherwise on its next connection, whose answer to a
/// `DoListRegistrations` still lists the registration.
///
/// A receiver must understand it, so that one that does not know it refuses the request, naming
/// it.
#[derive(Model, EnvelopeComponent, Debug, Clone, PartialEq)]
#[model(label = "registry.do_end_registration")]
#[model(id = "Q55Fv41JNnDCT-zJBSgFLerYGdYX53HGCmzD2C83onw")]
#[model(no_ffi)]
#[envelope(must_understand)]
pub struct DoEndRegistration {}
