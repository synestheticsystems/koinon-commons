use ankurah::{envelope::EnvelopeComponent, Model};
use ankurah_commons::property::Timestamp;

/// A registry's word that every permission slip of an application, issued before a moment, is
/// void, as when an API key that asked for some was revoked, or the membership of a person who did
/// was closed: how a slip ends before its expiry without any station waiting on a registry. The
/// application is the `ForApplication` beside it. It has its authority from the connection it
/// comes on: a station takes it only from the registry the application came through, keeps the
/// latest such moment for each application, and from then on honours none of its slips issued
/// before it. The registry gives the moment later than every slip it issued before, as
/// `PermissionSlip` sets out, so that a slip issued in the same millisecond, before the voiding, is
/// void with the rest. The registry voids the good slips with the bad, and their parties ask for
/// new ones.
///
/// A request that holds it, beside `Request` and its `ForApplication`, tells a station of the
/// voiding, and needs no tag of its own, since this component is only ever the news of one; it
/// comes from a registry alone, down its connection, and is answered with `Response` alone. A
/// registry's `KeptRegistrations` carries the latest voiding of each registration's application
/// too, for a station that was not connected when it was made. A receiver must understand it, since
/// a station that dropped it would go on honouring void slips.
// OPEN: voiding reaches beyond a registry's part of joining applications and keeping the lists of
// stations and their devices. Lean: keep it, since without it nothing but each station's person
// can end a slip that a leaked credential asked for before it lapses.
#[derive(Model, EnvelopeComponent, Debug, Clone, PartialEq)]
#[model(label = "authority.slips_voided")]
#[model(id = "B0TYD6vLCA8Vb3I-wJMLXoiokXB9blRy3ItfSfK5ABo")]
#[model(no_ffi)]
#[envelope(must_understand)]
pub struct SlipsVoided {
    #[property(id = "VSkLbKcFQEPERWoc0FY7HMoUURhJ3JOJPiptcDXUuNo")]
    #[envelope(number = 0)]
    pub before: Timestamp,
}
