//! What a station is told of an application's authority, which it applies as soon as it hears it
//! and never waits on: the application's retirement, and a voiding of its slips; and a computer's
//! claim of an invitation, which makes the computer an application the station shares with.

mod claim_invitation;
mod retire;
mod void_slips;

pub use claim_invitation::*;
pub use retire::*;
pub use void_slips::*;
