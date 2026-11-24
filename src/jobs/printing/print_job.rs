use ankurah::{
    envelope::{ContentRef, EnvelopeComponent},
    Model,
};

use super::Copies;
use crate::mime_type::MimeType;

/// What a print prints: the job's id, the document's format, how many copies, and the document,
/// by its hash and its length, which binds the document that follows a `DoPrint`'s envelope on its
/// stream, as the message's attachment. A `DoPrint` carries it, and a station's journal and a
/// registry's record of the job keep it. What it prints is set when the job is taken and never
/// changes: a repeat of the request is compared with it, and a request under the same id that
/// asks for something else is refused.
///
/// The document itself is not kept. A registry passes a job on only to a station connected to it
/// at that moment, and refuses the job when the station is not, so it never holds a document to
/// send later; the caller repeats the request once the station is back.
#[derive(Model, EnvelopeComponent, Debug, Clone, PartialEq)]
#[model(label = "jobs.print_job")]
#[model(id = "nQF-f7gcJIaahDZ02CJzwtanB7_YAW1t9NK1E0Ng-Nw")]
#[model(no_ffi)]
pub struct PrintJob {
    /// The sender's id for the job, by which a repeat of the request is the same job, however
    /// often it is sent and across restarts, so that a station prints it once. An id names one job
    /// within the authority the job comes under: the application, and the party too when the job
    /// comes under a permission slip, so that the same id from another application or another
    /// party names another job. A registry's gateway sends a job under the id of its own record of
    /// it, so that the ids of two of its callers never meet at a station.
    #[active_type(LWW)]
    #[property(id = "_Wh9DS3Ko5IWHPUD-LgOrAMvDFyYONjSDm3T_rMLZAU")]
    #[envelope(number = 0)]
    pub job_id: String,
    /// The document's MIME type, such as `application/pdf`, or `application/x-zpl2` for a label
    /// in the printer's own language. A job in a format its printer's `document_formats` do not
    /// list is refused.
    #[active_type(LWW)]
    #[property(id = "GSvYIeWQJj2N0WsQnFFj5xC5_RiVZrw3zm9G8GrZw38")]
    #[envelope(number = 1)]
    pub document_format: MimeType,
    #[property(id = "BAPEzYjSNDjjmq5ukHGsuuPdIaH3kRKd0iJaol9jmj8")]
    #[envelope(number = 2)]
    pub copies: Copies,
    /// The document, by its SHA-256 and its length. With the device, the job's id, the document's
    /// format and the copies, it tells a repeated request from a different job under the same id.
    /// A station takes the attachment that follows a `DoPrint` only if it has this length and this
    /// hash.
    #[active_type(LWW)]
    #[property(id = "SFG98ogiC83p3FOODD0SvG35w2AINnC5wSBPR48VtXw")]
    #[envelope(number = 3)]
    pub document: ContentRef,
}
