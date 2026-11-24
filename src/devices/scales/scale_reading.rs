use ankurah::{envelope::EnvelopeComponent, Model};

use super::{WeightStatus, WeightUnit, WeightValue};

/// What a scale read, in the unit it read in, and whether it had settled: the answer to a weigh,
/// which a registry passes on to the caller that asked. It is out of date as soon as it is read,
/// so a station keeps none; a log of weighs could.
#[derive(Model, EnvelopeComponent, Debug, Clone, PartialEq)]
#[model(label = "devices.scale_reading")]
#[model(id = "eDuEmy-xISXG7eDmP77yM_UfS6NCmn4JNkq6jZBxz-Q")]
#[model(no_ffi)]
pub struct ScaleReading {
    /// Whether the value can be taken as the weight on the scale.
    #[active_type(LWW)]
    #[property(id = "0PJuwvmyZiB2F5D1rN6IJ5KL4GP_BESAxU_HACH4bws")]
    #[envelope(number = 0)]
    pub status: WeightStatus,
    /// The weight in `unit`, as the scale gave it.
    #[property(id = "8yIFGTbkPpXspiNxBE2Jd0jLmLGnc14r_8blAQHx7rs")]
    #[envelope(number = 1)]
    pub value: WeightValue,
    #[active_type(LWW)]
    #[property(id = "uQUJbR1ZK1OvZMNppeTJwFYLbM4CbT8PlLNL5rWT28k")]
    #[envelope(number = 2)]
    pub unit: WeightUnit,
}
