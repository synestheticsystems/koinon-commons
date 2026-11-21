//! Who may ask a station for what, and how a request proves it. An application is a named key
//! pair that a station's person chose to share the station's devices with, and a request has its
//! authority from the application's key: signed whole by it, or under a permission slip it signed
//! for the party that sends the request. A station checks either by itself, with no registry,
//! relay or internet involved, so that a job reaches it from anyone its person trusts, from
//! wherever they are.
//!
//! Here are the application a request or a record is for, by its key; the body of a permission
//! slip; the restrictions on where, for what and until when an authority holds, which a receiver
//! must understand; the carriers of what an application's key signed, as the exact bytes it
//! signed; how an application's authority ends before it lapses; and the requests that bring a
//! station that news.

mod for_application;
mod permission_slip;
mod restrictions;
mod revocation;
mod signatures;

pub mod requests;

pub use for_application::*;
pub use permission_slip::*;
pub use restrictions::*;
pub use revocation::*;
pub use signatures::*;
