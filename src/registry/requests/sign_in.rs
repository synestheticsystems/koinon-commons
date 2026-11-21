use ankurah::Model;

use crate::codec::Minted;

/// Hands a registry the sign-in of a person who may register the station there, so that the
/// registry holds that person on the connection for a `Register`, until the connection closes.
/// It requires `IdTokenSignIn`.
///
/// It is answered with `SignedInPerson`, and a record for each scope and sub-scope the person may
/// register the station in, under its entity id, holding `Scope` and `Registrable`. Where the
/// person may register only in some of a scope's sub-scopes, that scope comes as a record holding
/// `Scope` alone, so that the station can show the sub-scopes under its name. A person who is a
/// member of no scope gets no record, and may name a `FirstScope` instead. A token the registry
/// does not take is refused as `credential_refused`, and the person signs in again.
///
/// A sender lists it as a component a receiver must understand, so that a receiver that does not
/// know it refuses it by name.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "registry.sign_in")]
#[model(id = "CBj7AMB51aZpXl977dpybnA9abGzoeM0drvDeIfW6Qo")]
#[model(no_ffi)]
pub struct SignIn {}
