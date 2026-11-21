use ankurah::{
    proto::{Attested, Event},
    Model,
};

use crate::codec::Minted;

/// The events of the sender's own records, parents before children, as Ankurah encodes them, so
/// that the receiver imports them unchanged: an event's id is the hash of what it holds, so only
/// the sender's own events give the receiver its records under the ids the sender gave them, with
/// the same history. A push of events carries it; the receiver imports the events, and keeps no
/// record of this component.
///
/// The events are encoded together, as one list, by bincode 1 with its default options, as
/// Ankurah's own messages encode them, and the codec never reads inside them. So the two ends of
/// a push build against the same revision of Ankurah.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "nodes.events")]
#[model(id = "aoHfYIvYtO1Q8Dl82g4dzSeu7pmFL5uwx0s40VaVmhc")]
#[model(no_ffi)]
pub struct Events {
    /// The events, in their encoding: [`Events::new`] makes it, and [`Events::read`] reads it.
    #[property(id = "Aj9eYwuz5OvLpRJpOdYZmt0l7c0in-BiVYTaQuQEg04")]
    pub events: Vec<u8>,
}

impl Events {
    /// `events` in the order given, which puts each event after its parents.
    pub fn new(events: &[Attested<Event>]) -> Self {
        Events { events: bincode::serialize(events).expect("bincode encodes every attested event") }
    }

    /// The events, in the order the sender gave them.
    pub fn read(&self) -> Result<Vec<Attested<Event>>, UnreadableEvents> {
        bincode::deserialize(&self.events).map_err(UnreadableEvents)
    }
}

/// Why the bytes of an [`Events`] are no list of events this build of Ankurah reads.
#[derive(Debug, thiserror::Error)]
#[error("the events do not read: {0}")]
pub struct UnreadableEvents(pub bincode::Error);

#[cfg(test)]
mod tests {
    use super::*;
    use ankurah::{
        proto::{AuthorId, BackendOperation, Clock, Membership, ModelId, Operation, OperationSet, SystemModel},
        EntityId,
    };

    /// A genesis and an update after it, by a named author.
    fn events() -> Vec<Attested<Event>> {
        let mut operations = OperationSet::default();
        operations.push(Operation::Backend { backend: "lww".into(), operations: vec![BackendOperation { diff: vec![1, 2, 3] }] });
        operations.push(Operation::Membership(Membership::Add(ModelId::EntityId(EntityId::from_bytes([3; 32])))));
        let created = Event::genesis_from_parts([7; 32], 1_794_600_000_000, operations);
        let mut later = OperationSet::default();
        later.push(Operation::Membership(Membership::Add(ModelId::System(SystemModel::Model))));
        let updated = Event::update(created.entity_id, Clock::singleton(&created), AuthorId::Id(EntityId::from_bytes([2; 32])), later);
        vec![Attested::from(created), Attested::from(updated)]
    }

    #[test]
    fn events_read_back_in_their_order_with_their_ids() {
        let events = events();
        let read = Events::new(&events).read().unwrap();
        assert_eq!(read, events);
        assert_eq!(
            read.iter().map(|event| event.payload.id()).collect::<Vec<_>>(),
            events.iter().map(|event| event.payload.id()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn bytes_that_are_no_list_of_events_do_not_read() {
        let mut bytes = Events::new(&events()).events;
        bytes.truncate(bytes.len() - 1);
        assert!(Events { events: bytes }.read().is_err());
        assert!(Events { events: vec![0xff; 8] }.read().is_err());
    }
}
