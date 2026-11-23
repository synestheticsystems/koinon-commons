use ankurah::Model;
use ankurah_commons::dimension::Dimension;
use serde::{Deserialize, Serialize};

use crate::codec::Minted;

/// A stock, by its name and size. On a printer's entity it is the stock loaded in the printer,
/// as a person last said, not as the printer senses it: after a change of stock it is right
/// again once someone says what the new stock is. On an entry of a registry's catalog it is a
/// stock the registry offers, from which a person picks what a printer is loaded with. The pick
/// is written whole on the printer's entity, name and size, and not as a reference to the
/// catalog, so that it reads the same on every server the station is signed in to.
#[derive(Model, Minted, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[model(label = "devices.media")]
#[model(id = "EJ_qLii0tXiKK-WbYNpqjl_vvCTVdMMrX8bJ7DyWzm0")]
#[model(no_ffi)]
pub struct Media {
    /// What people call the stock, such as "4x6".
    #[active_type(LWW)]
    #[property(id = "9dnXlG1Q2K1x4H1mYICkq2UrJmB8mp2TIKVvk1OxwN4")]
    pub name: String,
    /// Greater than zero.
    #[property(id = "lqS3yX7vFCJc_N_HmhKCg8I9MB_ozt96ZO9ItJif_f8")]
    pub width: Dimension,
    /// Greater than zero. None for continuous stock, which has no fixed length.
    #[active_type(LWW)]
    #[property(id = "wZeCZXsndsBshsjjYdOC0MOXA4qiUWyxWJAz7u7h7rg")]
    pub height: Option<Dimension>,
}
