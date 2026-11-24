use ankurah::{envelope::EnvelopeComponent, property::Ref, Model};
use serde::{Deserialize, Serialize};

use super::Scope;

/// The scope that the record on this entity belongs to, and its sub-scope when it belongs to one.
/// One component for every kind of record a registry places, so that an access filter reads the
/// same two properties whatever the record is. A registration carries the placement the station is
/// to serve in, as the registry then writes it.
///
/// A person, a key or a station placed in a scope as a whole reaches every one of its sub-scopes:
/// the person or the key acts in each, and the station serves each. One placed in a sub-scope
/// reaches that sub-scope alone.
#[derive(Model, EnvelopeComponent, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[model(label = "registry.placement")]
#[model(id = "OEr6KwWBmwnonz9izlHd_dFmH2FnZfnAxObTUp2kY0g")]
#[model(no_ffi)]
pub struct Placement {
    /// A scope that is no sub-scope.
    #[property(id = "PIW1N5qmOlvJsuynUxNENXH_6C13iz20UauTdcineo0")]
    #[envelope(number = 0)]
    pub scope: Ref<Scope>,
    /// One of `scope`'s own sub-scopes. None on a record that belongs to the scope as a whole.
    #[active_type(LWW)]
    #[property(id = "aotgkkATlNgIz0fhHPFekA-IOnSe2nBMxUedwf_InXA")]
    #[envelope(number = 1)]
    pub sub_scope: Option<Ref<Scope>>,
}
