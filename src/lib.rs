//! Definitions of the Ankurah models that Koinon programs share, so that two programs, or
//! two nodes, mean the same thing by "a device" or "a station". A layer on the Ankurah
//! Commons, in its shape: one module per namespace, one file per model.
//!
//! `devices` holds what a station shares with every server it is signed in to: its own
//! record, the printers and scales it offers, and the stock a printer is loaded with; and
//! what a scale reads. `nodes` holds what a node keeps about another node it is signed in
//! to, and the events of its own records that it hands one. `jobs` holds what a station is
//! asked to do on its devices and where each job stands, `catalog` what a registry knows of
//! printers, `registry` a registry's scopes and how a station registers there, and
//! `authority` who may ask a station for what and how a request proves it. `replies` holds what
//! any reply may say of its request. A namespace's `requests` module holds the components that
//! ask for something of the records it holds.
//!
//! `mime_type` holds the MIME types by which a document names its format and a printer the
//! formats it takes, `public_key` the keys components name nodes and applications by, `name`
//! the names people give what they make, and `component_ids` the lists by which one component
//! names others.
//!
//! The models' definitions are minted once and kept beside their structs, as events in the
//! text form of the Ankurah Commons' `akev`. `seed` loads them into a durable node, so that
//! the models take the minted ids there; `namespaces` lists the models the mint tool mints.
//!
//! `codec` holds envelopes, in which nodes send each other the components of these models and
//! of models built on them, keyed by the same minted ids. It holds no models of its own.

// So that the code `#[derive(Minted)]` generates, which names this crate from outside, also
// compiles on the crate's own models.
extern crate self as koinon_commons;

pub mod authority;
pub mod catalog;
pub mod codec;
pub mod component_ids;
pub mod devices;
pub mod jobs;
pub mod mime_type;
pub mod name;
pub mod namespaces;
pub mod nodes;
pub mod public_key;
pub mod registry;
pub mod replies;
mod seed;
mod stored_forms;

pub use seed::{seed, SeedError};
