//! What a station is asked to do on its devices, and where each job stands: the device a request
//! or a job is for, what a person sees of a job, what a print prints, and the job's latest
//! attempt; and the operations that ask for a print or a weigh. A request carries these
//! components, and a station's journal and a registry's record of a job can keep the same ones.

mod attempts;
mod for_device;
mod job_label;
mod printing;

pub mod operations;

pub use attempts::*;
pub use for_device::*;
pub use job_label::*;
pub use printing::*;
