use ankurah::{envelope::EnvelopeComponent, Model};

/// Asks a station to read one of its scales. Beside `Request`, it requires `ForDevice`, the scale,
/// and an authority, as `authority` sets out: the request signed whole by its application, or an
/// `AuthToken` presenting a permission slip. It names no job, so a repeat reads the scale again,
/// and no attachment follows it.
///
/// It is answered with `Response` and a `ScaleReading`. It is refused as `unsupported` on a device
/// that is no scale, and as `device_unavailable` when the scale cannot be read now, as when it is
/// not plugged in.
///
/// A receiver must understand it, so that one that does not know it refuses the request, naming
/// it.
#[derive(Model, EnvelopeComponent, Debug, Clone, PartialEq)]
#[model(label = "jobs.do_weigh")]
#[model(id = "OJJyxUs1Nrc-c6PfhzsitowdMhmdNqm0n4SvwMrqEBM")]
#[model(no_ffi)]
#[envelope(must_understand)]
pub struct DoWeigh {}
