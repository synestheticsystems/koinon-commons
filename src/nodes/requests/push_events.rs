use ankurah::Model;

use crate::codec::Minted;

/// Hands a registry the events of the station's own records, which the registry imports, while
/// it holds the station registered for an application. It requires `Events`, and is answered
/// with `Done` once every event is imported: an import commits every event or none, so the answer
/// has nothing more to say. Refused as `not_signed_in` while the registry holds the station
/// registered for no application, and as `not_authorized`, whole, when an event belongs to no
/// record of the station's own: its `Station` record, and the records of the devices that name it.
///
/// A station whose events take more than one frame pushes them in several requests, each event
/// after its parents. A sender lists it as a component a receiver must understand.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "nodes.push_events")]
#[model(id = "_Bgsf10qslz5g-303UwPDTx-aJdNgtqhLOMIWDKg2u0")]
#[model(no_ffi)]
pub struct PushEvents {}
