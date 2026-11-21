//! What a registry knows about printers, which it tells the stations registered with it, so that
//! it can recognise a new printer without any station being updated: how it recognises a kind of
//! printer by its driver, and a station's request for the catalog. The stock a registry offers is
//! the devices' `Media`, one record for each stock. The records a registry starts with are its
//! own, and not the Commons'.

mod printer_fingerprint;

pub mod requests;

pub use printer_fingerprint::*;
