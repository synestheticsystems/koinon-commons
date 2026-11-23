//! Permission slips, by which an application's authority reaches a program that does not hold the
//! application's key: the body of a slip, and the party it is for.

mod party;
mod permission_slip;

pub use party::*;
pub use permission_slip::*;
