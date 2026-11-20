use ankurah::Model;
use serde::{Deserialize, Serialize};

/// That the device on this entity prints, as the computer's print system describes it.
/// What it is loaded with is `Media`, on the same entity.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "devices.printer")]
#[model(id = "tdb_BWOhavVLjE6j3fg5AxGEvVYxVpy7vSx3IdvCxzE")]
#[model(no_ffi)]
pub struct Printer {
    /// The name of the driver that the print system uses for it, such as "Zebra ZPL Label
    /// Printer": on macOS what the queue reports as its make and model, on Windows the name
    /// of its driver. Every queue has one, though it may name a generic driver and say
    /// nothing of the printer. A server can work out the printer's own language from it.
    #[active_type(LWW)]
    pub driver: String,
    /// Whether the print system renders a PDF for it.
    pub accepts_pdf: bool,
}
