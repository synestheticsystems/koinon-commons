use ankurah::Model;

use crate::codec::Minted;

/// The sender's id for a job, by which a repeat of the request is the same job, however often it
/// is sent and across restarts. An id names one job within the authority the job comes under: the
/// application, and the party too when the job comes under a permission slip, so that the same id
/// from another application or another party names another job. A registry's gateway sends a job
/// under the id of its own record of it, so that the ids of two of its callers never meet at a
/// station.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "jobs.job_id")]
#[model(id = "2u-GoGD0-mhb2KoSffEbjnH1cu5tYB7g8UDU7SMcQaA")]
#[model(no_ffi)]
pub struct JobId {
    /// The id, as its sender wrote it.
    #[active_type(LWW)]
    #[property(id = "z9RWzSYmCBxf3rJcn41u6mpwS10ZSy1zIMA9MH0nyoE")]
    pub job: String,
}
