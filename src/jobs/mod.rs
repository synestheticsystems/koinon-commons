//! What a station is asked to do on its devices, and where each job stands: the device a request
//! or a job is for, the sender's id for the job and what a person sees of it, what a print prints
//! and how its document follows, and the job's latest attempt. A request carries these components,
//! and a station's journal and a registry's record of a job can keep the same ones.

mod attempts;
mod for_device;
mod job_id;
mod job_label;
mod printing;

pub mod requests;

pub use attempts::*;
pub use for_device::*;
pub use job_id::*;
pub use job_label::*;
pub use printing::*;
