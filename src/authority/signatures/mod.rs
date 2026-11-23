//! What an application's key signed, carried as the exact bytes it signed: a request signed whole,
//! the permission slip a party's request comes under, the key's retirement, and the key's
//! endorsement of a registry's hosting of the application. Each carrier carries an envelope's bytes,
//! its signer's key and the signature; a receiver opens it, which verifies the signature over the
//! bytes as they arrived and only then reads the envelope from those same bytes, and acts on
//! nothing else.

mod presented_slip;
mod signature;
mod signed;
mod signed_hosting;
mod signed_request;
mod signed_retirement;

pub use presented_slip::*;
pub use signature::*;
pub use signed::*;
pub use signed_hosting::*;
pub use signed_request::*;
pub use signed_retirement::*;
