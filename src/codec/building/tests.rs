//! Building an envelope: the bytes its layout gives, in the shortest forms, and the same bytes
//! whatever order a sender adds its components in or its struct declares their properties in;
//! and what no envelope is built with, refused naming the component, the property or the entity
//! it concerns.

use ankurah::{property::Json, Model, Value};
use serde_json::json;

use super::*;
use crate::codec::{
    tests::{decoded, formats, label, one, pointer, property, size, Deadline, Formats, Label, Marker, Size},
    Properties, ReadError,
};

/// An id, or an entity id, as an envelope holds it: a byte string of 32 bytes, here each the byte
/// `repeated`.
fn id(repeated: u8) -> Vec<u8> {
    [&[0x58, 0x20][..], &[repeated; 32]].concat()
}

// The fixtures' ids are their bytes repeated: Marker's 02, Size's 03 and its width's 31,
// Deadline's 04 and its until's 41, Pointer's 06 and its target's 61.
#[test]
fn an_envelope_is_written_as_its_layout_says_in_the_shortest_forms() {
    let mut named = RecordBuilder::new(Some(EntityId::from_bytes([5; 32])));
    named.add(&Marker {}).unwrap();
    let mut unnamed = RecordBuilder::new(None);
    unnamed.add(&pointer()).unwrap();
    let mut envelope = EnvelopeBuilder::new();
    envelope.add_must_understand(&Deadline { until: 1 }).unwrap().add(&Size { height: None, ..size() }).unwrap();
    envelope.add_record(named).unwrap().add_record(unnamed).unwrap();

    let expected = [
        // An array of three, the first the components a receiver must understand: Deadline.
        [vec![0x83, 0x81], id(0x04)].concat(),
        // The envelope's own components in the order of their ids: Size, with its width and
        // without its height, then Deadline.
        [vec![0xa2], id(0x03), vec![0xa1], id(0x31), vec![0xfb], 101.6f64.to_be_bytes().to_vec()].concat(),
        [id(0x04), vec![0xa1], id(0x41), vec![0x01]].concat(),
        // Two records: entity 05's, of a Marker with no properties, and one that names no entity,
        // of a Pointer to entity 09.
        [vec![0x82, 0x82], id(0x05), vec![0xa1], id(0x02), vec![0xa0]].concat(),
        [vec![0x82, 0xf6, 0xa1], id(0x06), vec![0xa1], id(0x61), id(0x09)].concat(),
    ]
    .concat();
    assert_eq!(envelope.encode(), expected);
}

// A sender adds components in the order its code reaches them, and anyone who compares or hashes
// the envelope sees one envelope.
#[test]
fn the_same_components_make_the_same_bytes_in_whatever_order_they_are_added() {
    let mut record = RecordBuilder::new(None);
    record.add(&formats()).unwrap().add_must_understand(&Deadline { until: 1 }).unwrap();
    let mut forward = EnvelopeBuilder::new();
    forward.add(&label()).unwrap().add(&size()).unwrap().add_must_understand(&Marker {}).unwrap().add_record(record).unwrap();

    let mut record = RecordBuilder::new(None);
    record.add_must_understand(&Deadline { until: 1 }).unwrap().add(&formats()).unwrap();
    let mut backward = EnvelopeBuilder::new();
    backward.add_record(record).unwrap().add_must_understand(&Marker {}).unwrap().add(&size()).unwrap().add(&label()).unwrap();

    assert_eq!(forward.encode(), backward.encode());
}

/// Label as another sender's struct declares it, with its fields in the reverse order of their
/// properties' ids.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "tests.label")]
#[model(id = "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE")]
#[model(no_ffi)]
pub struct ReorderedLabel {
    #[active_type(LWW)]
    #[property(id = "ExMTExMTExMTExMTExMTExMTExMTExMTExMTExMTExM")]
    pub note: Option<String>,
    #[property(id = "EhISEhISEhISEhISEhISEhISEhISEhISEhISEhISEhI")]
    pub copies: i64,
    #[active_type(LWW)]
    #[property(id = "ERERERERERERERERERERERERERERERERERERERERERE")]
    pub name: String,
}

#[test]
fn a_component_makes_the_same_bytes_in_whatever_order_its_struct_declares_its_fields() {
    let Label { name, copies, note } = label();
    assert_eq!(one(&ReorderedLabel { note, copies, name }), one(&label()));
}

// A component a receiver must understand is so wherever the envelope holds it: it is listed once,
// however many times it was added so, and also when it was added plainly elsewhere.
#[test]
fn the_must_understand_list_names_each_such_component_once_in_the_order_of_their_ids() {
    let mut first = RecordBuilder::new(None);
    first.add_must_understand(&Deadline { until: 1 }).unwrap().add_must_understand(&size()).unwrap();
    let mut second = RecordBuilder::new(None);
    second.add_must_understand(&Deadline { until: 2 }).unwrap();
    let mut envelope = EnvelopeBuilder::new();
    envelope.add(&Deadline { until: 3 }).unwrap().add_must_understand(&Marker {}).unwrap();
    envelope.add_record(first).unwrap().add_record(second).unwrap();

    // The envelope's array of three, then a list of three: Marker, Size and Deadline.
    let listed = [vec![0x83, 0x83], id(0x02), id(0x03), id(0x04)].concat();
    assert_eq!(&envelope.encode()[..listed.len()], &listed[..]);
}

// The records are a list in the order the sender gives, such as a catalog's, and records that
// name no entity are each a record of their own.
#[test]
fn records_keep_the_order_they_were_added_in() {
    let entity = |repeated: u8| Some(EntityId::from_bytes([repeated; 32]));
    let entities = [entity(9), None, entity(1), None];
    let mut envelope = EnvelopeBuilder::new();
    for named in entities {
        let mut record = RecordBuilder::new(named);
        record.add(&Marker {}).unwrap();
        envelope.add_record(record).unwrap();
    }
    let bytes = envelope.encode();
    let read: Vec<Option<EntityId>> = decoded(&bytes).records().map(|record| record.entity()).collect();
    assert_eq!(read, entities);
}

// The same component in the envelope's own components and in each of its records is no component
// held twice, and a component refused leaves the envelope as it was.
#[test]
fn a_component_added_twice_to_the_envelope_or_to_one_record_is_refused() {
    let mut envelope = EnvelopeBuilder::new();
    envelope.add(&label()).unwrap();
    let refused = envelope.add_must_understand(&Label { copies: 1, ..label() }).unwrap_err();
    assert!(matches!(refused, EncodeError::DuplicateComponent { component } if component == Label::ID), "{refused}");
    let mut record = RecordBuilder::new(None);
    record.add_must_understand(&Deadline { until: 1 }).unwrap();
    let refused = record.add(&Deadline { until: 2 }).unwrap_err();
    assert!(matches!(refused, EncodeError::DuplicateComponent { component } if component == Deadline::ID), "{refused}");

    record.add(&label()).unwrap();
    let mut other = RecordBuilder::new(None);
    other.add(&label()).unwrap();
    envelope.add_record(record).unwrap().add_record(other).unwrap();
    let bytes = envelope.encode();
    let received = decoded(&bytes);
    assert_eq!(received.components().read::<Label>().unwrap(), label());
    assert!(received.records().all(|record| record.components().has::<Label>()));
    assert_eq!(received.must_understand(), [Deadline::ID]);
}

#[test]
fn an_entity_given_a_second_record_is_refused() {
    let entity = EntityId::from_bytes([5; 32]);
    let mut envelope = EnvelopeBuilder::new();
    envelope.add_record(RecordBuilder::new(Some(entity))).unwrap();
    let refused = envelope.add_record(RecordBuilder::new(Some(entity))).unwrap_err();
    assert!(matches!(refused, EncodeError::DuplicateRecord { entity: twice } if twice == entity), "{refused}");
    assert_eq!(decoded(&envelope.encode()).records().len(), 1);
}

/// Label as a `Minted` impl written by hand might write it, with its name twice, which
/// `#[derive(Minted)]` never does.
#[derive(Model, Debug, Clone)]
#[model(label = "tests.label")]
#[model(no_ffi)]
pub struct TwiceNamedLabel {
    #[active_type(LWW)]
    pub name: String,
}

impl Minted for TwiceNamedLabel {
    const ID: MintedId = Label::ID;

    fn write_properties(&self, properties: &mut PropertyWriter) -> Result<(), EncodeError> {
        properties.write(property::<Label>("name"), &self.name)?;
        properties.write(property::<Label>("name"), &self.name)
    }

    fn read_properties(_: &Properties<'_>) -> Result<Self, ReadError> {
        unreachable!("the tests never read one")
    }
}

#[test]
fn a_component_that_writes_a_property_twice_is_refused_naming_the_property() {
    let name = property::<Label>("name");
    let refused = EnvelopeBuilder::new().add(&TwiceNamedLabel { name: "Pack bench 3".into() }).unwrap_err();
    let EncodeError::DuplicateProperty { component, property } = refused else { panic!("{refused}") };
    assert_eq!((component, property), (Label::ID, name));
}

/// A writer of `C`'s properties, as the code `#[derive(Minted)]` generates for `C` is given one.
fn writer<C: Minted>() -> PropertyWriter {
    PropertyWriter { component: C::ID, properties: Vec::new() }
}

/// A property type whose `Property` impl is wrong, as a test needs it to be: it declares string
/// values, and converts to an integer or not at all.
enum Faulty {
    Integer,
    Unconvertible,
}

impl Property for Faulty {
    const VALUE_TYPE: &'static str = "string";

    fn into_value(&self) -> Result<Option<Value>, PropertyError> {
        match self {
            Faulty::Integer => Ok(Some(Value::I64(2))),
            Faulty::Unconvertible => Err(PropertyError::InvalidValue { value: "unconvertible".into(), ty: "Faulty".into() }),
        }
    }

    fn from_value(_: Option<Value>) -> Result<Self, PropertyError> {
        unreachable!("the tests never read one")
    }
}

#[test]
fn a_value_of_another_type_than_its_property_declares_is_refused_naming_both_types() {
    let note = property::<Label>("note");
    let refused = writer::<Label>().write(note, &Faulty::Integer).unwrap_err();
    let EncodeError::WrongValueType { component, property, declared, produced } = refused else { panic!("{refused}") };
    assert_eq!((component, property, declared, produced), (Label::ID, note, ValueType::String, ValueType::I64));
}

#[test]
fn a_value_its_type_does_not_convert_is_refused_with_the_types_own_error() {
    let note = property::<Label>("note");
    let refused = writer::<Label>().write(note, &Faulty::Unconvertible).unwrap_err();
    let EncodeError::Conversion { component, property, error } = &refused else { panic!("{refused}") };
    assert_eq!((*component, *property), (Label::ID, note));
    assert_eq!(**error, PropertyError::InvalidValue { value: "unconvertible".into(), ty: "Faulty".into() });
}

// Ankurah's own `Json` holds JSON of any shape, where an envelope carries a plain value or a list
// of them.
#[test]
fn json_with_parts_is_refused_naming_the_property_that_holds_it() {
    let types = property::<Formats>("types");
    for parted in [json!({ "name": "4x6" }), json!(["application/pdf", ["application/vnd.cups-raw"]])] {
        let refused = writer::<Formats>().write(types, &Json(parted.clone())).unwrap_err();
        let EncodeError::JsonWithParts { component, property } = refused else { panic!("{parted}: {refused}") };
        assert_eq!((component, property), (Formats::ID, types), "{parted}");
    }
}

/// Label with a field that is neither stored nor sent: whether a list a person looks at shows it
/// selected.
#[derive(Model, Minted, Debug, Clone, PartialEq)]
#[model(label = "tests.label")]
#[model(id = "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE")]
#[model(no_ffi)]
pub struct SelectableLabel {
    #[active_type(LWW)]
    #[property(id = "ERERERERERERERERERERERERERERERERERERERERERE")]
    pub name: String,
    #[property(id = "EhISEhISEhISEhISEhISEhISEhISEhISEhISEhISEhI")]
    pub copies: i64,
    #[active_type(LWW)]
    #[property(id = "ExMTExMTExMTExMTExMTExMTExMTExMTExMTExMTExM")]
    pub note: Option<String>,
    #[model(ephemeral)]
    pub selected: bool,
}

#[test]
fn an_ephemeral_field_is_not_sent_and_reads_as_its_default() {
    let Label { name, copies, note } = label();
    let shown = SelectableLabel { name, copies, note, selected: true };
    let bytes = one(&shown);
    assert_eq!(bytes, one(&label()));
    let read: SelectableLabel = decoded(&bytes).components().read().unwrap();
    assert_eq!(read, SelectableLabel { selected: false, ..shown });
}
