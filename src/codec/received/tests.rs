//! Reading a received envelope: the bytes it arrived as, kept; the components a receiver
//! understands, read into their structs from the envelope's own components and from each record's,
//! required or optional; what it does not know, ignored; and what it requires and does not find,
//! or finds in another form, refused naming the component and the property.

use ankurah::{Model, Value, ValueType};
use ankurah_commons::dimension::Dimension;

use super::*;
use crate::codec::{
    tests::{decoded, formats, label, one, payload, pointer, property, size, Deadline, Formats, Label, Marker, Payload, Pointer, Size},
    EnvelopeBuilder, RecordBuilder,
};

// A signature over the envelope is checked against these bytes, so they are the ones that arrived
// and never bytes encoded again.
#[test]
fn a_received_envelope_keeps_the_very_bytes_it_was_decoded_from() {
    let bytes = one(&label());
    assert!(std::ptr::eq(decoded(&bytes).bytes(), bytes.as_slice()));
}

// One component for each kind of value a receiver meets: text, an integer, an optional text, a
// number its type checks, JSON, an entity id and bytes, and a component with no properties.
#[test]
fn every_component_reads_back_as_it_was_added_to_the_envelope_or_to_a_record() {
    let entity = EntityId::from_bytes([5; 32]);
    let mut record = RecordBuilder::new(Some(entity));
    record.add(&formats()).unwrap().add(&pointer()).unwrap().add(&payload()).unwrap();
    let deadline = Deadline { until: 1_767_225_600_000 };
    let mut envelope = EnvelopeBuilder::new();
    envelope.add(&label()).unwrap().add(&Marker {}).unwrap().add(&size()).unwrap().add(&deadline).unwrap().add_record(record).unwrap();
    let bytes = envelope.encode();

    let received = decoded(&bytes);
    let own: (Label, Marker, Size, Deadline) = received.components().read().unwrap();
    assert_eq!(own, (label(), Marker {}, size(), deadline));
    let records: Vec<(Option<EntityId>, (Formats, Pointer, Payload))> =
        received.records().map(|record| (record.entity(), record.components().read().unwrap())).collect();
    assert_eq!(records, [(Some(entity), (formats(), pointer(), payload()))]);
}

// A newer sender adds a component an older receiver has no struct for. To the codec that is a
// component the receiver does not read, and the receiver reads the rest as before.
#[test]
fn a_component_the_receiver_does_not_read_is_ignored_and_still_named_among_the_ids() {
    let mut envelope = EnvelopeBuilder::new();
    envelope.add(&payload()).unwrap().add(&label()).unwrap();
    let bytes = envelope.encode();
    let received = ReceivedEnvelope::decode(&bytes, &Limits::default(), &[]).unwrap();
    let components = received.components();
    assert_eq!(components.read::<Label>().unwrap(), label());
    assert_eq!(components.ids().collect::<Vec<_>>(), [Label::ID, Payload::ID]);
    assert!(components.has::<Payload>());
    assert!(!components.has::<Size>());
}

/// Label as a newer sender declares it, with a property this receiver's Label does not declare.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "tests.label")]
#[model(id = "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE")]
#[model(no_ffi)]
pub struct NewerLabel {
    #[active_type(LWW)]
    #[property(id = "ERERERERERERERERERERERERERERERERERERERERERE")]
    pub name: String,
    #[property(id = "EhISEhISEhISEhISEhISEhISEhISEhISEhISEhISEhI")]
    pub copies: i64,
    #[active_type(LWW)]
    #[property(id = "ExMTExMTExMTExMTExMTExMTExMTExMTExMTExMTExM")]
    pub note: Option<String>,
    #[active_type(LWW)]
    #[property(id = "FBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQ")]
    pub shelf: String,
}

#[test]
fn a_property_the_receivers_struct_does_not_declare_is_ignored() {
    let Label { name, copies, note } = label();
    let bytes = one(&NewerLabel { name, copies, note, shelf: "B4".into() });
    assert_eq!(decoded(&bytes).components().read::<Label>().unwrap(), label());
}

// A record's components are its own: one the envelope holds beside the record is not the
// record's.
#[test]
fn a_required_component_that_is_absent_is_refused_naming_it_and_an_optional_one_reads_as_none() {
    let mut record = RecordBuilder::new(None);
    record.add(&label()).unwrap();
    let mut envelope = EnvelopeBuilder::new();
    envelope.add(&size()).unwrap().add_record(record).unwrap();
    let bytes = envelope.encode();
    let received = decoded(&bytes);
    let in_the_record = received.records().next().unwrap().components();
    let refused = in_the_record.read::<(Label, Size)>().unwrap_err();
    assert!(matches!(refused, ReadError::MissingComponent { component } if component == Size::ID), "{refused}");
    assert_eq!(in_the_record.read::<(Label, Option<Size>)>().unwrap(), (label(), None));
    assert_eq!(received.components().read::<(Option<Label>, Size)>().unwrap(), (None, size()));
}

/// Size as a careless sender declares it: each property optional and of a looser type, so that it
/// may leave out the width, give any number for it, or give text for the height.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "tests.size")]
#[model(id = "AwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwM")]
#[model(no_ffi)]
pub struct LooseSize {
    #[property(id = "MTExMTExMTExMTExMTExMTExMTExMTExMTExMTExMTE")]
    pub width: Option<f64>,
    #[active_type(LWW)]
    #[property(id = "MjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjI")]
    pub height: Option<String>,
}

// An optional component the envelope holds is read whole or refused, never taken as absent.
#[test]
fn a_required_property_that_is_absent_is_refused_naming_it_and_an_optional_one_reads_as_none() {
    let bytes = one(&LooseSize { width: None, height: None });
    let width = property::<Size>("width");
    let refused = decoded(&bytes).components().read::<Option<Size>>().unwrap_err();
    let ReadError::MissingProperty { component, property } = refused else { panic!("{refused}") };
    assert_eq!((component, property), (Size::ID, width));
    let unmeasured = Size { height: None, ..size() };
    assert_eq!(decoded(&one(&unmeasured)).components().read::<Size>().unwrap(), unmeasured);
}

// So that an id in a log can be found in the source, where the struct's attributes write it.
#[test]
fn a_read_error_names_each_id_as_the_source_writes_it() {
    let bytes = one(&LooseSize { width: None, height: None });
    let message = decoded(&bytes).components().read::<Size>().unwrap_err().to_string();
    for id in ["AwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwM", "MTExMTExMTExMTExMTExMTExMTExMTExMTExMTExMTE"] {
        assert!(message.contains(id), "{message}");
    }
}

// An optional property is left out to say nothing, so one present in another form is refused,
// never read as nothing.
#[test]
fn a_value_in_another_form_is_refused_naming_its_property_even_when_the_property_is_optional() {
    let bytes = one(&LooseSize { width: Some(101.6), height: Some("tall".into()) });
    let height = property::<Size>("height");
    let refused = decoded(&bytes).components().read::<Size>().unwrap_err();
    let ReadError::MalformedValue { component, property, problem } = &refused else { panic!("{refused}") };
    assert_eq!((*component, *property), (Size::ID, height));
    assert!(matches!(problem, ValueProblem::UnexpectedType { expected: ValueType::F64, found: "a text string" }), "{problem}");
}

// A width below zero is refused as a stored record's would be, by Dimension's own rule.
#[test]
fn a_value_its_types_rule_refuses_is_refused_with_the_error_a_stored_records_read_gives() {
    let bytes = one(&LooseSize { width: Some(-1.0), height: None });
    let width = property::<Size>("width");
    let refused = decoded(&bytes).components().read::<Size>().unwrap_err();
    let ReadError::MalformedValue { component, property, problem: ValueProblem::Refused(error) } = &refused else { panic!("{refused}") };
    assert_eq!((*component, *property), (Size::ID, width));
    assert_eq!(**error, Dimension::from_value(Some(Value::F64(-1.0))).unwrap_err());
}

// A receiver names once, when it decodes an envelope, every component it understands, so that a
// handler may read what tells it what it was sent, then the rest, each read naming only what it
// takes.
#[test]
fn understanding_is_checked_when_the_envelope_is_decoded_and_not_by_each_read() {
    let mut envelope = EnvelopeBuilder::new();
    envelope.add(&label()).unwrap().add_must_understand(&Deadline { until: 1 }).unwrap();
    let bytes = envelope.encode();
    let refused = ReceivedEnvelope::decode(&bytes, &Limits::default(), &[Label::ID]).unwrap_err();
    assert_eq!(refused, DecodeError::NotUnderstood { components: vec![Deadline::ID] });

    let received = ReceivedEnvelope::decode(&bytes, &Limits::default(), &[Deadline::ID]).unwrap();
    assert_eq!(received.must_understand(), [Deadline::ID]);
    assert_eq!(received.components().read::<Label>().unwrap(), label());
    assert_eq!(received.components().read::<(Deadline, Option<Marker>)>().unwrap(), (Deadline { until: 1 }, None));
}

// A receiver of the envelope's own components alone would drop a component a record holds, so one
// the envelope lists as must-understand there is refused, named with those the receiver does not
// understand at all; a component a record holds that is not listed so is read past.
#[test]
fn a_receiver_of_the_envelopes_own_components_refuses_one_a_record_holds_that_it_must_understand() {
    let mut record = RecordBuilder::new(None);
    record.add_must_understand(&Deadline { until: 1 }).unwrap().add(&size()).unwrap();
    let mut envelope = EnvelopeBuilder::new();
    envelope.add(&label()).unwrap().add_must_understand(&Marker {}).unwrap().add_record(record).unwrap();
    let bytes = envelope.encode();

    let refused = ReceivedEnvelope::decode_own_components(&bytes, &Limits::default(), &[Label::ID, Deadline::ID]).unwrap_err();
    assert_eq!(refused, DecodeError::NotUnderstood { components: vec![Marker::ID, Deadline::ID] });
    let refused = ReceivedEnvelope::decode_own_components(&bytes, &Limits::default(), &[Label::ID, Marker::ID, Deadline::ID]).unwrap_err();
    assert_eq!(refused, DecodeError::NotUnderstood { components: vec![Deadline::ID] });
    assert!(ReceivedEnvelope::decode(&bytes, &Limits::default(), &[Marker::ID, Deadline::ID]).is_ok());

    let mut record = RecordBuilder::new(None);
    record.add(&Deadline { until: 1 }).unwrap();
    let mut envelope = EnvelopeBuilder::new();
    envelope.add_must_understand(&Marker {}).unwrap().add_record(record).unwrap();
    let bytes = envelope.encode();
    let received = ReceivedEnvelope::decode_own_components(&bytes, &Limits::default(), &[Marker::ID]).unwrap();
    assert_eq!(received.components().read::<Marker>().unwrap(), Marker {});
}

// A Label envelope holds twelve items: its array, the empty list, the components' map, Label's
// id and its properties' map, three properties each an id and a value, and the records' array.
#[test]
fn an_envelope_is_decoded_within_the_limits_its_receiver_gives() {
    let bytes = one(&label());
    let at_the_limit = Limits { max_items: 12, ..Limits::default() };
    assert!(ReceivedEnvelope::decode(&bytes, &at_the_limit, &[]).is_ok());
    let below = Limits { max_items: 11, ..Limits::default() };
    assert!(matches!(ReceivedEnvelope::decode(&bytes, &below, &[]), Err(DecodeError::TooManyItems { limit: 11, .. })));
}
