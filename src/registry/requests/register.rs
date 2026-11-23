use ankurah::Model;

use crate::codec::Minted;

/// Registers the station for an application, where the person whose sign-in the registry accepted
/// on this connection chose: the person's yes to sharing the devices on the station's computer
/// with the application. It requires `ScopeAssignment`, which names the station's `Station`
/// record; `Placement`, a scope or sub-scope of the sign-in's answer, or else `FirstScope`; and
/// `ForApplication`, the application, which a sender lists as a component a receiver must
/// understand, so that a registration for one application is never taken for one for all. A
/// registration that carries both a `Placement` and a `FirstScope` is refused as `bad_request`.
///
/// It is answered with the application's `ForApplication` and the `Registration` under which the
/// registry now keeps the station registered for it, a grant written for this registration: from
/// then on the registry's hello lists the registration, it passes the application's gateway jobs
/// down this connection, and it lists the station and its devices to the application's callers
/// in the place chosen. The station records its own trust in the application only once it has
/// this answer, and keeps the `Registration` on its record of the application. It is refused as
/// `not_signed_in` on a connection where the registry accepted no sign-in, as `not_found` when
/// the registry hosts no such application, and as `not_authorized` when the person may not
/// register a station where it asks, or when the registry keeps a `Station` record under this id
/// whose key is not the one the connection proves.
///
/// A registry keeps a station registered for each application at most once at a time. A repeat of
/// the registration that registered it, by the same person, for the same station record and in
/// the same place, is answered with the same `Registration`, and any other registration for the
/// same application is refused as `not_authorized`. So a station whose person removed the
/// application and added it again first sends its `StopSharing` under the earlier grant, and only
/// then registers anew, under a new grant, from whose moment the application's slips are
/// honoured.
///
/// A sender lists it as a component a receiver must understand, so that a receiver that does not
/// know it refuses it by name.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "registry.register")]
#[model(id = "qfoBT30BMtbpfYnzGa8RFDN7SgapUFZ39jusKkJnMys")]
#[model(no_ffi)]
pub struct Register {}
