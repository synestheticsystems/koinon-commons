use ankurah::Model;

use crate::codec::Minted;

/// Asks a station to print a document on one of its printers. It requires `ForDevice`, the
/// printer; `JobId`, the sender's id for the job; `PrintJob`, what to print; and
/// `DocumentFollows`, with the document after the envelope on the stream. It takes `JobLabel`,
/// which the station lists the job by. It requires an authority too: it travels inside a
/// `SignedRequest`, or beside a `PresentedSlip`.
///
/// It is answered with the job's `JobAttempt`, with a `JobFailure` beside it when the attempt
/// failed. The station records the job before it hands it to the printer. A repeat of a job it
/// sent prints nothing a second time, and is answered with the job's attempt; a repeat of one
/// whose latest attempt failed is tried once more; and a request under the job's id that asks for
/// something else, another device, format, number of copies or document, is refused as
/// `bad_request`. It is refused as `unsupported` on a device that is no printer, or for a document
/// in a format the printer's `document_formats` do not list, and as `device_unavailable` when the
/// printer cannot take work now.
///
/// A sender lists it as a component a receiver must understand, so that a receiver that does not
/// know it refuses it by name.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "jobs.print")]
#[model(id = "wI6UU04AwBLGIsIPuehq0feR4bANXBUcHvf9Ixs4H6Y")]
#[model(no_ffi)]
pub struct Print {}
