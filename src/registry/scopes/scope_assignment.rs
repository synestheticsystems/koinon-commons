use ankurah::{envelope::EnvelopeComponent, property::Ref, Model};
use serde::{Deserialize, Serialize};

use crate::devices::Station;

/// That a station's devices serve the scope its `Placement` names, on this registry. A station
/// may have several. A registration carries the assignment the station asks for, naming its
/// `Station` record, so that the registry can assign the station before it has pushed its records.
///
/// Who assigned it is the Ankurah Commons' `Created`, and its end `Closed`, on the same entity.
#[derive(Model, EnvelopeComponent, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[model(label = "registry.scope_assignment")]
#[model(id = "-jryf5idnQSkfFnbllipSPw2VR08tuKGq9IF8ZGRQUA")]
#[model(no_ffi)]
pub struct ScopeAssignment {
    #[property(id = "YBlIuqRgFN1koyPU1ZGFOKle-Cs1jXH7R62InanIWYM")]
    #[envelope(number = 0)]
    pub station: Ref<Station>,
}
