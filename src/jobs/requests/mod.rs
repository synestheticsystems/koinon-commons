//! What a station is asked to do on one of its devices. A request is one of these, with the facts
//! it acts on in the other components of `jobs`, and an authority from `authority`.

mod print;
mod weigh;

pub use print::*;
pub use weigh::*;
