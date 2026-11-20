use ankurah::Model;
use serde::{Deserialize, Serialize};

/// That the device on this entity weighs. A reading is not recorded: it carries its own
/// unit, and is out of date as soon as it is read.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "devices.scale")]
#[model(id = "7-rJXFa3T5o25tnMu7h-o8lIw3ERmG1jW2Q-rSXM-Og")]
#[model(no_ffi)]
pub struct Scale {}
