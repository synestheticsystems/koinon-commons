use ankurah::{envelope::EnvelopeComponent, Model};

/// Registers the station for an application, where the person whose sign-in the registry accepted
/// on this connection chose: the person's yes to sharing the devices on the station's computer
/// with the application. Beside `Request`, it requires `ScopeAssignment`, which names the
/// station's `Station` record; `Placement`, a scope or sub-scope of the sign-in's answer, or else
/// `FirstScope`; and `ForApplication`, the application, which a receiver must understand, so that
/// a registration for one application is never taken for one for all. A registration that carries
/// both a `Placement` and a `FirstScope`, or neither, is refused as `bad_request`.
///
/// It is answered with `Response`, the application's `ForApplication` and the `Registration` under
/// which the registry now keeps the station registered for it, a grant written for this
/// registration: from then on the registry's answer to a `DoListRegistrations` lists the
/// registration, it passes the application's gateway jobs down this connection, and it lists the
/// station and its devices to the application's callers in the place chosen. The station records
/// its own trust in the application only once it has this answer, and keeps the `Registration` on
/// its record of the application. It is refused as `not_signed_in` on a connection where the
/// registry accepted no sign-in, as `not_found` when the registry hosts no such application, and as
/// `not_authorized` when the person may not register a station where it asks, or when the registry
/// keeps a `Station` record under this id whose key is not the one the connection proves.
///
/// A registry keeps a station registered for each application at most once at a time. A repeat of
/// the registration that registered it, by the same person, for the same station record and in
/// the same place, is answered with the same `Registration`, and any other registration for the
/// same application is refused as `not_authorized`. So a station whose person removed the
/// application and added it again first sends its `DoEndRegistration` under the earlier grant, and
/// only then registers anew, under a new grant, from whose moment the application's slips are
/// honoured.
///
/// A station of Pylon 0.1.0 registers with no `ForApplication`, for no one application.
///
/// A receiver must understand it, so that one that does not know it refuses the request, naming
/// it.
#[derive(Model, EnvelopeComponent, Debug, Clone, PartialEq)]
#[model(label = "registry.do_register")]
#[model(id = "OjcLq131kyXCH8sPy3eB7TfXpP5rlAcS7u6SAQr97a0")]
#[model(no_ffi)]
#[envelope(must_understand)]
pub struct DoRegister {}
