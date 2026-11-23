use ankurah::Model;
use serde::{Deserialize, Serialize};

use super::{Copies, DocumentHash};
use crate::{codec::Minted, mime_type::MimeType};

/// What a print prints: the document's format, how many copies, and the document's hash, which
/// binds the document that follows the request. A print carries it, and a station's journal and a
/// registry's record of the job keep it. What it prints is set when the job is taken and never
/// changes: a repeat of the request is compared with it, and a request under the same id that
/// asks for something else is refused.
///
/// The document itself is not kept. A registry passes a job on only to a station connected to it
/// at that moment, and refuses the job when the station is not, so it never holds a document to
/// send later; the caller repeats the request once the station is back.
#[derive(Model, Minted, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[model(label = "jobs.print_job")]
#[model(id = "nQF-f7gcJIaahDZ02CJzwtanB7_YAW1t9NK1E0Ng-Nw")]
#[model(no_ffi)]
pub struct PrintJob {
    /// The document's MIME type, such as `application/pdf`, or `application/x-zpl2` for a label
    /// in the printer's own language. A job in a format its printer's `document_formats` do not
    /// list is refused.
    #[active_type(LWW)]
    #[property(id = "GSvYIeWQJj2N0WsQnFFj5xC5_RiVZrw3zm9G8GrZw38")]
    pub document_format: MimeType,
    #[property(id = "BAPEzYjSNDjjmq5ukHGsuuPdIaH3kRKd0iJaol9jmj8")]
    pub copies: Copies,
    /// With the device, the document's format and the copies, it tells a repeated request from a
    /// different job under the same id. A station takes the document that follows a print only
    /// if its bytes hash to it.
    #[active_type(LWW)]
    #[property(id = "Z7r75Ma_rNFuTBAwbvGmB3GlALeAzz7CGFpkVlWMxc0")]
    pub document_hash: DocumentHash,
}

#[cfg(test)]
mod tests {
    use super::*;

    // The JSON form a print job had in Pylon's own model, before the Commons held it, field by
    // field, so that what Pylon's code reads and writes of one reads as before.
    #[test]
    fn a_print_job_is_written_in_json_as_pylons_own_model_wrote_one() {
        let json = serde_json::json!({ "document_format": "application/x-zpl2", "copies": 2, "document_hash": "ab".repeat(32) });
        let print = PrintJob {
            document_format: MimeType::new("application/x-zpl2").unwrap(),
            copies: Copies::new(2).unwrap(),
            document_hash: DocumentHash::from_bytes([0xab; 32]),
        };
        assert_eq!(serde_json::to_value(&print).unwrap(), json);
        assert_eq!(serde_json::from_value::<PrintJob>(json).unwrap(), print);
    }
}
