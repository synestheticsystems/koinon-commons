//! The operations a station is asked to perform on one of its devices, each by an empty tag that
//! names it. A request holds the Ankurah Commons' `Request`, one of these, the facts it acts on
//! from the other components of `jobs`, and an authority from `authority`.

mod do_print;
mod do_weigh;

pub use do_print::*;
pub use do_weigh::*;
