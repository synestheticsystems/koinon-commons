//! What a node keeps about the other nodes it works with, and the events of its own records that
//! it hands them.

mod events;
mod node_link;

pub mod operations;

pub use events::*;
pub use node_link::*;
