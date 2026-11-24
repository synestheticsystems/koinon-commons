//! What a station asks its registry about its registrations, each by an empty tag that names the
//! operation. A person's sign-in needs no tag: `IdTokenSignIn`, which holds it, names it.

mod do_end_registration;
mod do_list_registrations;
mod do_register;

pub use do_end_registration::*;
pub use do_list_registrations::*;
pub use do_register::*;
