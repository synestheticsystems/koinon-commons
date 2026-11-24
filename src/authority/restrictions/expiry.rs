use ankurah::{envelope::EnvelopeComponent, Model};
use ankurah_commons::property::Timestamp;

/// The moment after which a permission slip or a signed request is no longer honoured, so that a
/// copy that leaks lapses by itself, with no station waiting on anyone to end it. Every slip and
/// every signed request has it.
// OPEN: whether an expiry belongs in the Ankurah Commons, for any record that lapses, beside its
// `Deferred`, the opposite moment. Lean: here for now, beside the authority it bounds, and the
// general question put to the Ankurah Commons.
#[derive(Model, EnvelopeComponent, Debug, Clone, PartialEq)]
#[model(label = "authority.expiry")]
#[model(id = "khmTklrBcgxzZEdATzyvhRvt_9HDLPPS8DYVyoQX_Ls")]
#[model(no_ffi)]
#[envelope(must_understand)]
pub struct Expiry {
    #[property(id = "L4Egrh_tkS7SE2EQcSa6pu2kPSP8TqnXV01GIckHwlQ")]
    #[envelope(number = 0)]
    pub expires: Timestamp,
}

impl Expiry {
    /// How far a station's clock and a signer's may disagree, in milliseconds, when the station
    /// compares a moment the signer wrote with its own: a minute, as a registry allows an ID
    /// token's times. A station allows it only where it compares its own clock with another's: two
    /// moments one issuer wrote by its one clock, such as a slip's issue and its expiry, are
    /// compared exactly.
    pub const CLOCK_SKEW_MS: i64 = 60_000;

    /// Whether the moment has passed at `now`, by the station's clock, allowing for the drift of
    /// clocks. Compared in 128 bits, so that no moment a signer writes overflows.
    pub fn has_passed(&self, now: Timestamp) -> bool {
        i128::from(now.unix_millis()) - i128::from(Self::CLOCK_SKEW_MS) > i128::from(self.expires.unix_millis())
    }

    /// Whether the moment lies more than `most_ms` after `from`, compared in 128 bits, so that no
    /// moment a signer writes overflows: how a station tells an expiry further off than it honours
    /// an authority for, from a slip's issue or from now.
    pub fn lies_beyond(&self, from: Timestamp, most_ms: i64) -> bool {
        i128::from(self.expires.unix_millis()) - i128::from(from.unix_millis()) > i128::from(most_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expiry(millis: i64) -> Expiry {
        Expiry { expires: Timestamp::from_unix_millis(millis) }
    }

    #[test]
    fn an_expiry_passes_only_once_the_drift_of_clocks_is_past_it() {
        let now = Timestamp::from_unix_millis(1_794_600_000_000);
        assert!(!expiry(1_794_600_000_000).has_passed(now));
        assert!(!expiry(1_794_600_000_000 - Expiry::CLOCK_SKEW_MS).has_passed(now));
        assert!(expiry(1_794_600_000_000 - Expiry::CLOCK_SKEW_MS - 1).has_passed(now));
    }

    #[test]
    fn no_moment_a_signer_writes_overflows_the_comparison() {
        assert!(!expiry(i64::MAX).has_passed(Timestamp::from_unix_millis(i64::MIN)));
        assert!(expiry(i64::MIN).has_passed(Timestamp::from_unix_millis(i64::MAX)));
    }

    #[test]
    fn an_expiry_lies_beyond_a_span_only_once_it_is_further_off_and_never_overflows() {
        let from = Timestamp::from_unix_millis(1_794_600_000_000);
        assert!(!expiry(1_794_600_000_000 + 600_000).lies_beyond(from, 600_000));
        assert!(expiry(1_794_600_000_000 + 600_001).lies_beyond(from, 600_000));
        assert!(expiry(i64::MAX).lies_beyond(Timestamp::from_unix_millis(i64::MIN), i64::MAX));
        assert!(!expiry(i64::MIN).lies_beyond(Timestamp::from_unix_millis(i64::MAX), 0));
    }
}
