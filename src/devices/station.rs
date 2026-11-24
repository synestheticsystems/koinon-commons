use ankurah::{envelope::EnvelopeComponent, Model};
use serde::{Deserialize, Serialize};

use crate::public_key::PublicKey;

/// A node that offers the printers and scales on its computer to the servers it is signed in
/// to, such as an installed copy of Pylon: what a server lists as a station, and what each of
/// its devices names.
///
/// The station makes this record once and is the only one to change it. Its retirement is
/// the Ankurah Commons' `Closed`, on the same entity.
#[derive(Model, EnvelopeComponent, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[model(label = "devices.station")]
#[model(id = "x7iVWAhMXd77hZaS-CRE598fanpmtDia7M_pQI-fKpU")]
#[model(no_ffi)]
pub struct Station {
    /// The key of the station's iroh endpoint, which each of its connections proves, so that a
    /// server recognises the station by it. It never changes: a node set up anew, under a new
    /// key, is another station.
    #[active_type(LWW)]
    #[property(id = "YEir7jUuQgbsYGJQY1cXRWdIFJMf3rgvtaotfuuraoE")]
    #[envelope(number = 0)]
    pub public_key: PublicKey,
    /// What a person called it, such as "Pack bench 3".
    #[active_type(LWW)]
    #[property(id = "sIfYWZ7pLs7btNnecBf5brIZ0y_hkp1sWTaOWDcmXbU")]
    #[envelope(number = 1)]
    pub name: String,
}
