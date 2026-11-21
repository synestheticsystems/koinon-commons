//! A registry's scopes, which stations serve them, and which nodes may act on it; and how a
//! station signs a person in there and registers for an application, and learns what it is
//! registered for. A registry is a server that joins applications and the stations that share
//! their devices with them, and keeps the lists of both.
// OPEN: whether the registry's scopes and grants belong in the Commons rather than in Pylon's own
// model, where they were first declared. Lean: yes, the namespace whole, with their ids, since
// the sign-in's answer and a registration carry them, and a namespace split across two crates
// could give two models one label.

mod id_token_sign_in;
mod node_grant;
mod registered_for;
mod scopes;
mod signed_in_person;

pub mod requests;

pub use id_token_sign_in::*;
pub use node_grant::*;
pub use registered_for::*;
pub use scopes::*;
pub use signed_in_person::*;
