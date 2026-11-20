//! Definitions of the Ankurah models that Koinon programs share, so that two programs, or
//! two nodes, mean the same thing by "a device" or "a station". A layer on the Ankurah
//! Commons, in its shape: one module per namespace, one file per model.
//!
//! `devices` holds what a station shares with every server it is signed in to: its own
//! record, the printers and scales it offers, and the stock a printer is loaded with.
//! `nodes` holds what a node keeps about another node it is signed in to.
//! `mime_type` holds the MIME types by which a document names its format and a printer the
//! formats it takes.
//!
//! The models' definitions are minted once and kept beside their structs, as events in the
//! text form of the Ankurah Commons' `akev`. `seed` loads them into a durable node, so that
//! the models take the minted ids there; `namespaces` lists the models the mint tool mints.

pub mod devices;
pub mod mime_type;
pub mod namespaces;
pub mod nodes;
mod seed;

pub use seed::{seed, SeedError};
