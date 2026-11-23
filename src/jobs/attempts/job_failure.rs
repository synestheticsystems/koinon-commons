use ankurah::Model;

use crate::codec::Minted;

/// How a job's latest attempt failed: beside a `JobAttempt` whose state is failed, and beside no
/// other. A failed attempt is no refusal: the station took the job, and the failure is the
/// outcome of its attempt.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "jobs.job_failure")]
#[model(id = "u-Oiv5qGhvh6PuR6PSPIzOyWUTxrqthkXqUKSCM5hS8")]
#[model(no_ffi)]
pub struct JobFailure {
    /// One word in snake case, for a program to act on, such as [`JobFailure::DEVICE_FAILED`]. A
    /// reader that does not know the word takes the failure as one it cannot act on, and shows the
    /// message.
    #[active_type(LWW)]
    #[property(id = "IKdwbbsNDF8JSE2NxyULnEnzOF0Y4jxiAmpVcf0wwOE")]
    pub code: String,
    /// What the device, or the system software in front of it, said of the failure.
    #[active_type(LWW)]
    #[property(id = "HhJ6UDg08sH5EzdI0iWqzHs495XXQlpyy-Q1C0w67ho")]
    pub message: String,
}

impl JobFailure {
    /// The device, or the system software in front of it, refused the job.
    pub const DEVICE_FAILED: &'static str = "device_failed";
    /// The device, or the system software in front of it, did not answer in time.
    pub const DEVICE_TIMEOUT: &'static str = "device_timeout";
    /// The station stopped before an outcome was recorded, so that nobody knows whether the job
    /// printed.
    pub const INTERRUPTED: &'static str = "interrupted";
}
