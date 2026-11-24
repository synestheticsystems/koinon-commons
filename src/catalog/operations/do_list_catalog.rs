use ankurah::{envelope::EnvelopeComponent, Model};

/// Asks a registry what it knows of printers and their stock, so that a person can pick what a
/// printer is loaded with and a station can tell which formats a printer takes. Beside `Request`,
/// it requires nothing. It is answered with `Response` and the registry's `Catalog`.
///
/// A receiver must understand it, so that one that does not know it refuses the request, naming
/// it.
#[derive(Model, EnvelopeComponent, Debug, Clone, PartialEq)]
#[model(label = "catalog.do_list_catalog")]
#[model(id = "gKrP-icPbro4dd0-EBlm3asf5eUOiM9MUQeM0ARCWcY")]
#[model(no_ffi)]
#[envelope(must_understand)]
pub struct DoListCatalog {}
