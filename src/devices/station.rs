use ankurah::Model;
use serde::{Deserialize, Serialize};

/// A node that offers the printers and scales on its computer to the servers it is signed in
/// to, such as an installed copy of Pylon: what a server lists as a station, and what each of
/// its devices names.
///
/// The station makes this record once and is the only one to change it. Its retirement is
/// the Ankurah Commons' `Closed`, on the same entity.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "devices.station")]
#[model(id = "x7iVWAhMXd77hZaS-CRE598fanpmtDia7M_pQI-fKpU")]
#[model(no_ffi)]
pub struct Station {
    /// The public key of the station's Ankurah node. A server recognises the station by it.
    /// It never changes: a node set up anew, under a new key, is another station.
    // OPEN: its text form follows Ankurah's node keys, which have not landed.
    #[active_type(LWW)]
    pub public_key: String,
    /// What a person called it, such as "Pack bench 3".
    #[active_type(LWW)]
    pub name: String,
}
