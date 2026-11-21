use ankurah::Model;
use serde::{Deserialize, Serialize};

use crate::{codec::Minted, mime_type::MimeTypes};

/// That the device on this entity prints, as the computer's print system describes it.
/// What it is loaded with is `Media`, on the same entity.
#[derive(Model, Minted, Debug, Clone, Serialize, Deserialize)]
#[model(label = "devices.printer")]
#[model(id = "tdb_BWOhavVLjE6j3fg5AxGEvVYxVpy7vSx3IdvCxzE")]
#[model(no_ffi)]
pub struct Printer {
    /// The name of the driver that the print system uses for it, such as "Zebra ZPL Label
    /// Printer": on macOS what the queue reports as its make and model, on Windows the name
    /// of its driver. Every queue has one, though it may name a generic driver and say
    /// nothing of the printer. A server can work out the printer's own language from it.
    #[active_type(LWW)]
    #[property(id = "VNc_b0EhyMd-LXZQbmvgYe9j1WUG6SS3qvdd2vknhvc")]
    pub driver: String,
    /// The formats of document the print system accepts for it, as MIME types, in the order
    /// the print system lists them, such as `application/pdf`, or `application/vnd.cups-raw`
    /// for bytes passed to the printer untouched. A caller chooses what to send by them, and a
    /// station refuses a document in a format they do not list.
    #[active_type(LWW)]
    #[property(id = "tUouhhdXl2SBfr3rG1wh6cm55yFPq38CvEBta6hyImY")]
    pub document_formats: MimeTypes,
}
