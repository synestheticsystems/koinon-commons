//! Permission slips, by which an application's authority reaches a program that does not hold the
//! application's key: the body of a slip, the party it is for, and the checks every station makes
//! of a slip before it acts on a request under it.

mod party;
mod permission_slip;
mod slip_checks;

pub use party::*;
pub use permission_slip::*;
pub use slip_checks::*;
