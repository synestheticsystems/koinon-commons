use ankurah::Model;

use crate::codec::Minted;

/// What a person sees of a job in a list: what its sender called it, and who asked for it, both
/// in the sender's own terms and never checked, since a station acts on nothing they say. A sender
/// with nothing to call a job leaves this out, and the station lists the job by its id.
// OPEN: whether a station takes a print without a label, where Pylon 0.1.0 requires a name on
// every print. Lean: it does, and lists the job by its id.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "jobs.job_label")]
#[model(id = "MoUpS9rhxNWBQV2TreJntKFbtesJPb2P118tZRRL_DE")]
#[model(no_ffi)]
pub struct JobLabel {
    /// What the sender called the job, such as "Order 1001".
    #[active_type(LWW)]
    #[property(id = "Gl_2LLylSjpIRLA1AQUvTteZ4yDHSnr0iFEJJ4PA9pY")]
    pub name: String,
    /// Who asked for the job, such as a user of the sender's own system.
    #[active_type(LWW)]
    #[property(id = "x3Wn_d3qGuDnWFrUkjQSlsNMHo0QhRdiM9U70a24d1o")]
    pub asked_by: Option<String>,
}
