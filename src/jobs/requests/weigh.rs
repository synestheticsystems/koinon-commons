use ankurah::Model;

use crate::codec::Minted;

/// Asks a station to read one of its scales. It requires `ForDevice`, the scale, and an
/// authority: it travels inside a `SignedRequest`, or beside a `PresentedSlip`, never both, since a
/// slip inside a signed request is refused as `bad_request`. It names no job, so a repeat reads the
/// scale again.
///
/// It is answered with a `ScaleReading`. It is refused as `unsupported` on a device that is no
/// scale, and as `device_unavailable` when the scale cannot be read now, as when it is not plugged
/// in.
///
/// A sender lists it as a component a receiver must understand, so that a receiver that does not
/// know it refuses it by name.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "jobs.weigh")]
#[model(id = "QlMqS2Al-4AE3iuq859ivZ-Tq47zokgyDuKqv5wR4rQ")]
#[model(no_ffi)]
pub struct Weigh {}
