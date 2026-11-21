//! How an application's authority ends before it lapses, with no station waiting on anyone: its
//! key's retirement, which the key signs itself, and a registry's voiding of its permission slips.

mod retirement;
mod slips_voided;

pub use retirement::*;
pub use slips_voided::*;
