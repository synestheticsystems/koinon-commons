use ankurah::{EntityId, Model};
use ankurah_commons::property::Timestamp;

use crate::{authority::Expiry, codec::Minted};

/// The body of a permission slip: the application whose key signs the slip lets the slip's party
/// make the requests the slip allows, where and until the slip says. It is how an application's
/// authority reaches a program that does not hold the application's key, such as a program on a
/// warehouse's own network, while the key stays with whoever holds it, such as the application's
/// registry, which signs a slip when a person, or a caller acting for the application, asks for
/// one. It travels inside a `PresentedSlip`, whose signer is the application, beside the party it
/// is for and the restrictions that say where, for what and until when the slip is honoured:
///
/// - `Party`, required: who the slip is for, by the key their connection proves;
/// - `AtStations`, required: the stations at which the slip is honoured, which tell the party
///   where to dial;
/// - `AtDevices`, optional: the devices at those stations to which the slip narrows;
/// - `AllowsRequests`, required: the kinds of request the party may make;
/// - `Expiry`, required: until when it is honoured, no earlier than its issue and at most
///   [`PermissionSlip::MAX_LIFE_MS`] after it.
///
/// A station honours a slip only where every restriction it carries is met. A slip that lacks
/// `Party`, `AtStations`, `AllowsRequests` or `Expiry` is refused. A restriction that is optional
/// and absent sets no limit of its own, so a restriction added later is optional and listed as one
/// a receiver must understand: it can only narrow what a slip allows. The slip lists this
/// component, its party and every restriction so.
///
/// A station honours a slip while it shares its devices with the slip's application, and only one
/// issued no earlier than the moment the application's registry wrote the station's registration
/// for the application, which the person's latest yes made, nor than the moment that registry last
/// voided the application's slips (`SlipsVoided`). The registry gives those moments and the slip's
/// issue by its one clock, so the station compares them exactly; and it gives each of them later
/// than every one it gave before, taking the millisecond after the last one it gave whenever its
/// clock reads that one or an earlier one, as within one millisecond or after a restart, which
/// carries on from the last one it gave. So a slip issued before a registration or a voiding is
/// earlier than it, however close the two come, and the exact comparison tells them apart. A
/// station refuses a slip that says it was issued later than now, by more than the drift of clocks
/// `Expiry` allows, one whose expiry comes before its issue, and one that runs on for longer than a
/// station honours a slip for.
// OPEN: whether an application that a station's person paired with directly, with no registry,
// may sign slips. The rules above apply either way; a station's check of the slip's signer names
// the applications that may.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "authority.permission_slip")]
#[model(id = "sW4D0xcq9TVaWSu6kyop7gXpHdCQcBjzagQOBaa_LYs")]
#[model(no_ffi)]
pub struct PermissionSlip {
    /// When the slip was issued, by its issuer's clock.
    #[property(id = "qjlXIUr2HftQP9Kf6lwBKiguJcEiczvr1VijjU5lWx0")]
    pub issued: Timestamp,
    /// The slip's id, which no other slip of the application has: on a registry, the id of its
    /// record of the slip. A station's journal names by it the slip each job came under, so that a
    /// job can be traced to the slip and to whoever asked for it.
    #[property(id = "tcjLiCa1HVdnOEGYOGO8ycyPRiDc6wD57s23SN8wjvw")]
    pub slip_id: EntityId,
}

impl PermissionSlip {
    /// The longest a slip is honoured for, in milliseconds from its issue: thirty days. A station
    /// refuses a slip whose expiry lies further from its issue, so that a slip a station never
    /// hears voided still lapses within a month.
    pub const MAX_LIFE_MS: i64 = 30 * 24 * 60 * 60_000;

    /// Whether the slip says it was issued later than `now`, by the station's clock, by more than
    /// [`Expiry::CLOCK_SKEW_MS`]: the one comparison of the slip's issue with the station's own
    /// clock, which allows for the drift of the two clocks. Compared in 128 bits, so that no moment
    /// an issuer writes overflows the comparison.
    pub fn is_issued_after(&self, now: Timestamp) -> bool {
        i128::from(self.issued.unix_millis()) - i128::from(Expiry::CLOCK_SKEW_MS) > i128::from(now.unix_millis())
    }

    /// Whether the slip was issued before `moment`, a moment of the clock of the registry that
    /// issued it: when it wrote the station's registration for the slip's application, or last
    /// voided the application's slips. Compared exactly, since both moments are of one clock, which
    /// gives each moment later than every one before it, so that a slip issued before the
    /// registration it rests on never passes, however soon after a removal the person said yes
    /// again.
    pub fn is_issued_before(&self, moment: Timestamp) -> bool {
        self.issued < moment
    }

    /// Whether the slip runs on for longer than a station honours one for: whether `expiry`, the
    /// slip's own, lies more than [`PermissionSlip::MAX_LIFE_MS`] after its issue, both moments of
    /// its issuer's clock.
    pub fn runs_too_long(&self, expiry: &Expiry) -> bool {
        expiry.lies_beyond(self.issued, Self::MAX_LIFE_MS)
    }

    /// Whether `expiry`, the slip's own, comes before the slip's issue: a slip that lapses before
    /// it begins, which no issuer means, and which the drift of clocks that
    /// [`Expiry::has_passed`] allows would otherwise let a station honour for a minute. Compared
    /// exactly, since both moments are of the issuer's clock.
    pub fn expires_before_issue(&self, expiry: &Expiry) -> bool {
        expiry.expires < self.issued
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ISSUED: i64 = 1_794_600_000_000;

    fn slip() -> PermissionSlip {
        PermissionSlip { issued: Timestamp::from_unix_millis(ISSUED), slip_id: EntityId::from_bytes([30; 32]) }
    }

    fn at(millis: i64) -> Timestamp {
        Timestamp::from_unix_millis(millis)
    }

    #[test]
    fn a_slip_is_issued_after_now_only_beyond_the_drift_of_clocks() {
        assert!(!slip().is_issued_after(at(ISSUED)));
        assert!(!slip().is_issued_after(at(ISSUED - Expiry::CLOCK_SKEW_MS)));
        assert!(slip().is_issued_after(at(ISSUED - Expiry::CLOCK_SKEW_MS - 1)));
        assert!(!PermissionSlip { issued: at(i64::MIN), ..slip() }.is_issued_after(at(i64::MAX)));
        assert!(PermissionSlip { issued: at(i64::MAX), ..slip() }.is_issued_after(at(i64::MIN)));
    }

    // A registration written, or a voiding made, at the moment of the slip's issue leaves the slip
    // honoured; one a millisecond later does not.
    #[test]
    fn a_slip_is_issued_before_a_moment_of_its_issuers_clock_compared_exactly() {
        assert!(!slip().is_issued_before(at(ISSUED)));
        assert!(slip().is_issued_before(at(ISSUED + 1)));
        assert!(!slip().is_issued_before(at(ISSUED - 1)));
    }

    #[test]
    fn a_slip_runs_too_long_only_past_thirty_days_from_its_issue() {
        let expiring = |millis| Expiry { expires: at(millis) };
        assert!(!slip().runs_too_long(&expiring(ISSUED + PermissionSlip::MAX_LIFE_MS)));
        assert!(slip().runs_too_long(&expiring(ISSUED + PermissionSlip::MAX_LIFE_MS + 1)));
        assert!(PermissionSlip { issued: at(i64::MIN), ..slip() }.runs_too_long(&expiring(i64::MAX)));
        assert!(!slip().runs_too_long(&expiring(i64::MIN)));
    }

    // A slip may lapse at the moment of its issue, and not a millisecond before it.
    #[test]
    fn a_slip_expires_before_its_issue_only_when_its_expiry_is_earlier_than_its_issue() {
        let expiring = |millis| Expiry { expires: at(millis) };
        assert!(!slip().expires_before_issue(&expiring(ISSUED)));
        assert!(!slip().expires_before_issue(&expiring(ISSUED + 1)));
        assert!(slip().expires_before_issue(&expiring(ISSUED - 1)));
    }
}
