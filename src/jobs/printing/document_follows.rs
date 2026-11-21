use ankurah::Model;

use super::DocumentLength;
use crate::codec::Minted;

/// That a print's document follows the request's envelope on its stream, as exactly `length` bare
/// bytes, so that a station checks the request's authority on the envelope alone, one small frame,
/// before it takes in anything larger from a sender it may not trust. The document's hash is the
/// print's `PrintJob::document_hash`: the station hashes the bytes as they arrive, and refuses as
/// `bad_request` a document whose hash is another. It refuses a length beyond the most it takes
/// before it reads any of the document.
///
/// A sender lists it as a component a receiver must understand, since it changes how the stream is
/// read: inside the signed envelope of a signed print, and beside the request under a slip.
// OPEN: whether a registry's gateway prints send their document after the frame too, as the
// direct path's do. Lean: yes, so that one path reads every print, checks its signature before
// any document arrives, and needs no frame of 64 MiB.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "jobs.document_follows")]
#[model(id = "KHr2sIEoCMDMH1g0YiIR-dsXoRc5vel8HGm1HqeaFX0")]
#[model(no_ffi)]
pub struct DocumentFollows {
    #[property(id = "E-OWlNv7oDUbTwDxfKoEV9vjRHpuF_bAxrGjvW2l-Kw")]
    pub length: DocumentLength,
}
