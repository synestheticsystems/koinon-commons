use ankurah::{
    envelope::{EntityRows, EnvelopeComponent},
    Model,
};

/// What a registry knows of printers and their stock: the answer to a `DoListCatalog`. A row for
/// each stock the registry offers, under its record's entity id, holds `Media`; then a row for each
/// `PrinterFingerprint`, under its record's entity id, in the registry's order, by which the first
/// that names a driver decides.
// OPEN: a stock in the catalog is the devices' `Media`, which a person's pick is copied into
// whole, rather than a model of the catalog's own with the same name, width and height. Lean:
// yes, since both mean a stock, and no record of a catalog's own model of stock exists yet.
#[derive(Model, EnvelopeComponent, Debug, Clone, PartialEq)]
#[model(label = "catalog.catalog")]
#[model(id = "mIYmHLiouWyCF0-Vf5hodBz8EiRfi6zwjx4n-wejljs")]
#[model(no_ffi)]
pub struct Catalog {
    /// A row for each stock, then a row for each printer fingerprint, in that order.
    #[active_type(LWW)]
    #[property(id = "n_MIrQ-5GbF1ZBSUiM2m37-BMwmo8K1eUUPC0gLj2xI")]
    #[envelope(number = 0)]
    pub entries: EntityRows,
}
