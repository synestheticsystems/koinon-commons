//! A registry's scopes, which stations serve them, and which nodes may act on it; and how a
//! station signs a person in there, registers for an application, and learns which registrations
//! the registry keeps for it. A registry is a server that joins applications and the stations that
//! share their devices with them, and keeps the lists of both.
// OPEN: whether the registry's scopes and grants belong in the Commons rather than in Pylon's own
// model, where they were first declared. Lean: yes, the namespace whole, with their ids, since the
// sign-in's answer and a registration carry them, and a namespace split across two crates could
// give two models one label. The other view keeps `NodeGrant` in Pylon's model, since a registry
// alone writes and reads a grant, and brings the scopes across alone; a `Registration` would then
// name its grant by its entity id, under the same minted id.

mod for_grant;
mod id_token_sign_in;
mod node_grant;
mod registration;
mod scopes;
mod signed_in_person;

pub mod requests;

pub use for_grant::*;
pub use id_token_sign_in::*;
pub use node_grant::*;
pub use registration::*;
pub use scopes::*;
pub use signed_in_person::*;
