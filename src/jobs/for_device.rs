use ankurah::{envelope::EnvelopeComponent, property::Ref, Model};

use crate::devices::Device;

/// The printer or scale that a request, or a job, is for: a print's or a weigh's, and a station's
/// or a registry's record of the job.
#[derive(Model, EnvelopeComponent, Debug, Clone, PartialEq)]
#[model(label = "jobs.for_device")]
#[model(id = "_P11X5Urc4p0vCN3OamJqR4HKtyCrg-W3MXcdpXr7TA")]
#[model(no_ffi)]
pub struct ForDevice {
    /// The device's `Device` record, which its own station made, so that its id names that
    /// station too.
    #[property(id = "JcVqbGhsU9ij-4pa2EoIzQi3OZPQTWL2TkXIL8VRZ90")]
    #[envelope(number = 0)]
    pub device: Ref<Device>,
}
