use ankurah::Model;
use ankurah_commons::property::Timestamp;

use super::AttemptState;
use crate::codec::Minted;

/// Where a job stands: its latest attempt, which is accepted, sent or failed, when that attempt
/// began, and when it ended. The answer to a print, which a registry passes on to the caller that
/// sent the job, and what a station's journal and a registry's record of the job say of it. A
/// failed attempt has a `JobFailure` beside it, which says how it failed.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "jobs.job_attempt")]
#[model(id = "XScVj8uOgAH6b4RWj_T6tcmDUoOiE12JHGYYal6all0")]
#[model(no_ffi)]
pub struct JobAttempt {
    #[active_type(LWW)]
    #[property(id = "AV_DeDhIlHezccpBom_tsaOlwlw7tj_-HT8RLMV-PkE")]
    pub state: AttemptState,
    /// When the latest attempt began.
    #[property(id = "IZxGoX-d9LWlifuBpbUiGWAYAA3uVvJcDrkI_B_ATqA")]
    pub accepted: Timestamp,
    /// When the latest attempt ended. None while it has no outcome.
    #[active_type(LWW)]
    #[property(id = "OLKshlCjy20CfAfCeMI4NzcTi5_cohvD5RW-HdW4V0A")]
    pub finished: Option<Timestamp>,
}
