use ankurah::{envelope::EnvelopeComponent, Model};

/// Asks a station to print a document on one of its printers. Beside `Request`, it requires
/// `ForDevice`, the printer, and `PrintJob`, what to print, whose `document` names the document
/// that follows the envelope on the stream, as the message's attachment; it takes `JobLabel`,
/// which the station lists the job by. It requires an authority, as `authority` sets out: the
/// request signed whole by its application, or an `AuthToken` presenting a permission slip.
///
/// The station checks the print's authority twice. It checks it on the envelope, before it reads
/// any of the attachment, so that a sender it does not trust costs it one envelope and nothing
/// more; it then reads exactly the length `PrintJob::document` gives, hashing the bytes as they
/// arrive, and refuses as `bad_request` an attachment of another length, one that ends early, or
/// one whose hash is another. It checks the authority again once the document has arrived, before
/// the job reaches the printer: a print whose application its person removed, or whose party they
/// blocked, while its document was arriving is refused then as `not_authorized`, and nothing is
/// printed.
///
/// It is answered with `Response` and the job's `JobAttempt`, with a `JobFailure` beside them when
/// the attempt failed. The station records the job before it hands it to the printer. A repeat of
/// a job it sent prints nothing a second time, and is answered with the job's attempt as recorded;
/// a repeat of one whose latest attempt failed is tried once more; and a request under the job's
/// id that asks for something else, another device, format, number of copies or document, is
/// refused as `bad_request`. It is refused as `unsupported` on a device that is no printer, or for
/// a document in a format the printer's `document_formats` do not list, and as
/// `device_unavailable` when the printer cannot take work now.
///
/// A receiver must understand it, so that one that does not know it refuses the request, naming
/// it.
#[derive(Model, EnvelopeComponent, Debug, Clone, PartialEq)]
#[model(label = "jobs.do_print")]
#[model(id = "0nEV9VeASXxHrHf28jwGlYPZ6TOghoNVvHnVnhlkQTM")]
#[model(no_ffi)]
#[envelope(must_understand)]
pub struct DoPrint {}
