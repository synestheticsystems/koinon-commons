//! What a station is told of an application's authority, which it applies as soon as it hears
//! it and never waits on: the application's retirement, and a voiding of its slips.

mod retire;
mod void_slips;

pub use retire::*;
pub use void_slips::*;
