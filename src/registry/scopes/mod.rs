//! A registry's scopes, where its records belong and its stations serve: the scopes themselves,
//! the placement of a record in one, a station's assignment to one, and what the sign-in's answer
//! and a registration say of where a station may be registered.

mod first_scope;
mod placement;
mod registrable;
mod scope;
mod scope_assignment;
mod scope_choices;

pub use first_scope::*;
pub use placement::*;
pub use registrable::*;
pub use scope::*;
pub use scope_assignment::*;
pub use scope_choices::*;
