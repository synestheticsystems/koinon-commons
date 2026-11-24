use ankurah::{envelope::EnvelopeComponent, Model};

/// Asks a registry to import the events of the station's own records, while it holds the station
/// registered for an application. Beside `Request`, it requires `Events`. It is answered with
/// `Response` alone once every event is imported: an import commits every event or none, so the
/// answer has nothing more to say. It is refused as `not_signed_in` while the registry holds the
/// station registered for no application, and as `not_authorized`, whole, when an event belongs to
/// no record of the station's own: its `Station` record, and the records of the devices that name
/// it.
///
/// A station whose events take more than one envelope sends them in several requests, each event
/// after its parents.
///
/// A receiver must understand it, so that one that does not know it refuses the request, naming
/// it.
#[derive(Model, EnvelopeComponent, Debug, Clone, PartialEq)]
#[model(label = "nodes.do_import_events")]
#[model(id = "sf6ZrWvoLdLtvrlpcojGEewv-B8Swzue8PxLCxg231I")]
#[model(no_ffi)]
#[envelope(must_understand)]
pub struct DoImportEvents {}
