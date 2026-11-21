use ankurah::Model;

use crate::codec::Minted;

/// Registers the station for an application, where the person whose sign-in the registry accepted
/// on this connection chose: the person's yes to sharing the devices on the station's computer
/// with the application. It requires `ScopeAssignment`, which names the station's `Station`
/// record; `Placement`, a scope or sub-scope of the sign-in's answer, or else `FirstScope`; and
/// `ForApplication`, the application, which a sender lists as a component a receiver must
/// understand, so that a registration for one application is never taken for one for all.
///
/// It is answered with `Done` once the registry holds the station registered for the application:
/// from then on the registry's hello lists the application, it passes the application's gateway
/// jobs down this connection, and it lists the station and its devices to the application's
/// callers in the place chosen. It is refused as `not_signed_in` on a connection where the
/// registry accepted no sign-in, as `not_found` when the registry hosts no such application, and
/// as `not_authorized` when the person may not register a station where it asks, or when the
/// registry holds a `Station` record under this id whose key is not the one the connection proves.
/// A repeat of the registration that registered the station, by the same person, for the same
/// station record and in the same place, is answered with `Done`, and any other registration for
/// the same application is refused as `not_authorized`.
///
/// A sender lists it as a component a receiver must understand, so that a receiver that does not
/// know it refuses it by name.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "registry.register")]
#[model(id = "qfoBT30BMtbpfYnzGa8RFDN7SgapUFZ39jusKkJnMys")]
#[model(no_ffi)]
pub struct Register {}
