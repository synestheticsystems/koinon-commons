//! What a print prints, and how its document reaches the station: after the envelope of the
//! request, on the same stream, bound by its hash.

mod copies;
mod document_follows;
mod document_hash;
mod document_length;
mod print_job;

pub use copies::*;
pub use document_follows::*;
pub use document_hash::*;
pub use document_length::*;
pub use print_job::*;
