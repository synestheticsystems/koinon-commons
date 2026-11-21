use ankurah::Model;

use crate::codec::Minted;

/// Asks a registry what it knows of printers and their stock, so that a person can pick what a
/// printer is loaded with and a station can tell which formats a printer takes. It is answered
/// with a record for each stock the registry offers, holding `Media`, then a record for each
/// `PrinterFingerprint`, in the registry's order, by which the first that names a driver decides.
///
/// A sender lists it as a component a receiver must understand, so that a receiver that does not
/// know it refuses it by name.
// OPEN: a stock in the catalog is the devices' `Media`, which a person's pick is copied into
// whole, rather than a model of the catalog's own with the same name, width and height. Lean:
// yes, since both mean a stock, and no record of a catalog's own model of stock exists yet.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "catalog.get_catalog")]
#[model(id = "KB2O0qtyH_Y98GpU1-LrfsSOSxclYXn7CItu5LPcInA")]
#[model(no_ffi)]
pub struct GetCatalog {}
