use ankurah::Model;
use ankurah_commons::dimension::Dimension;
use serde::{Deserialize, Serialize};

/// The stock loaded in the printer on this entity, as a person last said, not as the printer
/// senses it: after a change of stock it is right again once someone says what the new stock
/// is. It is written whole, name and size, and not as a reference to a list of stock, so that
/// it reads the same on every server the station is signed in to.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "devices.media")]
#[model(id = "EJ_qLii0tXiKK-WbYNpqjl_vvCTVdMMrX8bJ7DyWzm0")]
#[model(no_ffi)]
pub struct Media {
    /// What people call the stock, such as "4x6".
    #[active_type(LWW)]
    pub name: String,
    /// Greater than zero.
    pub width: Dimension,
    /// Greater than zero. None for continuous stock, which has no fixed length.
    #[active_type(LWW)]
    pub height: Option<Dimension>,
}
