//! What a registry knows about printers, which it tells the stations registered with it, so that
//! it can recognise a new printer without any station being updated: how it recognises a kind of
//! printer by its driver, a station's request for the catalog, and the catalog that answers it.
//! The stock a registry offers is the devices' `Media`, one record for each stock. The records a
//! registry starts with are its own, and not the Commons'.

// The mint finds a model's source by the model's name, and the catalog is this namespace's own.
#[allow(clippy::module_inception)]
mod catalog;
mod printer_fingerprint;

pub mod operations;

pub use catalog::*;
pub use printer_fingerprint::*;
