//! Envelopes a careless or hostile sender writes: each is refused before anything is read from it,
//! saying where, within the receiver's limits; and one written in another order, or in longer forms
//! than it needs, reads as the same components.

use super::*;
use crate::codec::{
    tests::{formats, label, pointer, size, Deadline, Formats, Label, Pointer, Size, UNDERSTOOD},
    EnvelopeBuilder, Minted, ReceivedEnvelope, RecordBuilder,
};

/// Ids of components and properties no receiver knows.
const A: [u8; 32] = [0xa0; 32];
const B: [u8; 32] = [0xb0; 32];
const P: [u8; 32] = [0x0a; 32];
const Q: [u8; 32] = [0x0b; 32];

/// Where the value of `holding`'s one property begins: after the envelope's head, the empty list of
/// components to understand, the components' head, A's id, its properties' head and P's id.
const VALUE_AT: usize = 1 + 1 + 1 + 34 + 1 + 34;

/// The head of an item of major type `major`, in its shortest form, written here apart from the
/// codec's own writer.
fn head(major: u8, argument: u64) -> Vec<u8> {
    let major = major << 5;
    match argument {
        0..=23 => vec![major | argument as u8],
        24..=0xff => vec![major | 24, argument as u8],
        0x100..=0xffff => [vec![major | 25], (argument as u16).to_be_bytes().to_vec()].concat(),
        0x1_0000..=0xffff_ffff => [vec![major | 26], (argument as u32).to_be_bytes().to_vec()].concat(),
        _ => [vec![major | 27], argument.to_be_bytes().to_vec()].concat(),
    }
}

/// The head of an item of major type `major`, in its longest form.
fn long(major: u8, argument: u64) -> Vec<u8> {
    [vec![major << 5 | 27], argument.to_be_bytes().to_vec()].concat()
}

fn id(bytes: [u8; 32]) -> Vec<u8> {
    [head(2, 32), bytes.to_vec()].concat()
}

/// A component's id, with each of its properties' ids and that property's value, already encoded.
type Held = ([u8; 32], Vec<([u8; 32], Vec<u8>)>);

/// A map of components in the order given, each property's value already encoded.
fn components(held: &[Held]) -> Vec<u8> {
    let mut out = head(5, held.len() as u64);
    for (component, properties) in held {
        out.extend(id(*component));
        out.extend(head(5, properties.len() as u64));
        for (property, value) in properties {
            out.extend(id(*property));
            out.extend(value);
        }
    }
    out
}

fn record(entity: Option<[u8; 32]>, components: Vec<u8>) -> Vec<u8> {
    [vec![0x82], entity.map_or(vec![0xf6], id), components].concat()
}

/// An envelope written item by item in the order given: the components listed as must-understand,
/// its own components as `components` writes them, and its records as `record` does.
fn envelope(must_understand: &[[u8; 32]], own: Vec<u8>, records: &[Vec<u8>]) -> Vec<u8> {
    let mut out = vec![0x83];
    out.extend(head(4, must_understand.len() as u64));
    for component in must_understand {
        out.extend(id(*component));
    }
    out.extend(own);
    out.extend(head(4, records.len() as u64));
    for record in records {
        out.extend(record);
    }
    out
}

/// An envelope of one component no receiver knows, holding one property whose value is `value`.
fn holding(value: Vec<u8>) -> Vec<u8> {
    envelope(&[], components(&[(A, vec![(P, value)])]), &[])
}

/// An envelope of one record, which holds what `holding` holds, so that the value comes last.
fn holding_in_a_record(value: Vec<u8>) -> Vec<u8> {
    envelope(&[], components(&[]), &[record(None, components(&[(A, vec![(P, value)])]))])
}

/// The index of `bytes` for a receiver that reads the components it understands wherever the
/// envelope holds them, as every test here reads one; `received`'s tests read one for a receiver
/// of the envelope's own components alone.
fn index(bytes: &[u8], limits: &Limits, understood: &[MintedId]) -> Result<Index, DecodeError> {
    super::index(bytes, limits, understood, Reading::Everywhere)
}

fn read(bytes: &[u8]) -> Result<Index, DecodeError> {
    index(bytes, &Limits::default(), &[])
}

fn refusal(bytes: &[u8]) -> DecodeError {
    read(bytes).unwrap_err()
}

/// Where the last item that is the id `bytes_of_id` begins in `bytes`.
fn last(bytes: &[u8], bytes_of_id: [u8; 32]) -> usize {
    bytes.windows(34).rposition(|window| window == id(bytes_of_id)).unwrap()
}

fn minted(bytes: [u8; 32]) -> MintedId {
    MintedId::from_bytes(bytes)
}

#[test]
fn every_envelope_cut_short_is_refused_as_truncated() {
    let mut record = RecordBuilder::new(Some(EntityId::from_bytes([5; 32])));
    record.add(&size()).unwrap().add_must_understand(&Deadline { until: 1 }).unwrap();
    let mut envelope = EnvelopeBuilder::new();
    envelope.add(&label()).unwrap().add(&formats()).unwrap().add_record(record).unwrap();
    let bytes = envelope.encode();
    assert!(index(&bytes, &Limits::default(), UNDERSTOOD).is_ok());
    for len in 0..bytes.len() {
        let refused = index(&bytes[..len], &Limits::default(), UNDERSTOOD).unwrap_err();
        assert!(matches!(refused, DecodeError::Truncated { .. }), "{len} of {}: {refused}", bytes.len());
    }
}

#[test]
fn bytes_after_the_envelope_are_refused() {
    let mut bytes = holding(vec![0x01]);
    let end = bytes.len();
    bytes.push(0x00);
    assert_eq!(refusal(&bytes), DecodeError::TrailingBytes { at: end });
}

// Each is refused where it stands, inside a component no receiver knows, so that an envelope one
// node accepts no other refuses for its form.
#[test]
fn what_no_envelope_holds_is_refused_even_in_a_component_no_receiver_knows() {
    let cases: [(&[u8], &str); 9] = [
        (&[0x9f, 0x01, 0xff], "an item of indefinite length"),
        (&[0xbf, 0xff], "an item of indefinite length"),
        (&[0x5f, 0x41, 0x00, 0xff], "an item of indefinite length"),
        (&[0x7f, 0x61, 0x61, 0xff], "an item of indefinite length"),
        (&[0xc1, 0x1a, 0x51, 0x4b, 0x67, 0xb0], "a tag"),
        (&[0xf7], "undefined"),
        (&[0xf0], "a simple value"),
        (&[0xff], "a break"),
        (&[0x1c], "a malformed head"),
    ];
    for (value, what) in cases {
        assert_eq!(refusal(&holding(value.to_vec())), DecodeError::Excluded { at: VALUE_AT, what }, "{value:02x?}");
        let within = [&[0x81][..], value].concat();
        assert_eq!(refusal(&holding(within)), DecodeError::Excluded { at: VALUE_AT + 1, what }, "{value:02x?} in an array");
    }
}

#[test]
fn text_that_is_not_utf8_is_refused_even_in_a_component_no_receiver_knows() {
    assert_eq!(refusal(&holding(vec![0x62, 0xc3, 0x28])), DecodeError::NotUtf8 { at: VALUE_AT });
}

#[test]
fn an_envelope_of_another_shape_is_refused_naming_what_an_envelope_holds_there() {
    let cases: Vec<(Vec<u8>, &str)> = vec![
        (vec![0x82, 0x80, 0xa0], ENVELOPE),
        (vec![0xa0], ENVELOPE),
        (vec![0x83, 0xa0, 0xa0, 0x80], LISTED),
        (vec![0x83, 0x80, 0x80, 0x80], COMPONENTS),
        ([&[0x83, 0x80, 0xa1][..], &id(A), &[0x80, 0x80]].concat(), PROPERTIES),
        ([&[0x83, 0x80, 0xa1, 0x58, 0x1f][..], &[0; 31], &[0xa0, 0x80]].concat(), ID),
        (vec![0x83, 0x81, 0x01, 0xa0, 0x80], ID),
        (vec![0x83, 0x80, 0xa0, 0xa0], RECORDS),
        (vec![0x83, 0x80, 0xa0, 0x81, 0x83, 0xf6, 0xa0, 0xa0], RECORD),
        (vec![0x83, 0x80, 0xa0, 0x81, 0x82, 0x41, 0x00, 0xa0], ENTITY),
        (vec![0x83, 0x80, 0xa0, 0x81, 0x82, 0x61, b'x', 0xa0], ENTITY),
    ];
    for (bytes, expected) in cases {
        let refused = refusal(&bytes);
        assert!(matches!(refused, DecodeError::NotAnEnvelope { expected: e, .. } if e == expected), "{bytes:02x?}: {refused}");
    }
}

#[test]
fn a_component_property_or_listing_held_twice_is_refused_where_it_recurs() {
    let twice = envelope(&[], components(&[(A, vec![]), (B, vec![]), (A, vec![])]), &[]);
    assert_eq!(refusal(&twice), DecodeError::DuplicateComponent { component: minted(A), at: last(&twice, A) });
    let twice = envelope(&[], components(&[(A, vec![(P, vec![0x01]), (Q, vec![0x02]), (P, vec![0x03])])]), &[]);
    assert_eq!(refusal(&twice), DecodeError::DuplicateProperty { component: minted(A), property: minted(P), at: last(&twice, P) });
    let twice = envelope(&[A, A], components(&[(A, vec![])]), &[]);
    assert_eq!(
        index(&twice, &Limits::default(), &[minted(A)]).unwrap_err(),
        DecodeError::MustUnderstandTwice { component: minted(A), at: 2 + 34 }
    );
    let absent = envelope(&[B], components(&[(A, vec![])]), &[]);
    assert_eq!(index(&absent, &Limits::default(), &[minted(B)]).unwrap_err(), DecodeError::MustUnderstandAbsent { component: minted(B) });
}

// A component may stand in several records, once in each, and records that name no entity are
// not one entity's.
#[test]
fn a_record_that_holds_a_component_twice_or_repeats_an_entity_is_refused() {
    let held = || components(&[(A, vec![])]);
    let twice = envelope(&[], components(&[]), &[record(None, components(&[(A, vec![]), (A, vec![])]))]);
    assert_eq!(refusal(&twice), DecodeError::DuplicateComponent { component: minted(A), at: last(&twice, A) });
    let repeated = envelope(&[], components(&[]), &[record(Some(B), held()), record(None, held()), record(Some(B), held())]);
    assert_eq!(refusal(&repeated), DecodeError::DuplicateRecord { entity: EntityId::from_bytes(B), at: last(&repeated, B) });
    let apart = envelope(&[], held(), &[record(Some(B), held()), record(None, held()), record(None, held()), record(Some(Q), held())]);
    let index = read(&apart).unwrap();
    let entities: Vec<Option<EntityId>> = index.records.iter().map(|record| record.entity).collect();
    assert_eq!(entities, [Some(EntityId::from_bytes(B)), None, None, Some(EntityId::from_bytes(Q))]);
}

// A key written twice is one key, however it is written: in a longer head, or as a float of
// another width. Each case gives a map with its second key's place in it.
#[test]
fn a_map_that_holds_a_key_twice_is_refused_however_the_key_is_written() {
    let cases: [(Vec<u8>, usize); 5] = [
        (vec![0xa2, 0x61, b'k', 0x01, 0x61, b'k', 0x02], 4),
        (vec![0xa2, 0x01, 0x00, 0x18, 0x01, 0x00], 3),
        (vec![0xa2, 0xf9, 0x3c, 0x00, 0x00, 0xfb, 0x3f, 0xf0, 0, 0, 0, 0, 0, 0, 0x00], 5),
        (vec![0x81, 0xa2, 0x41, 0x00, 0xf6, 0x41, 0x00, 0xf6], 5),
        (vec![0xa2, 0xf6, 0x00, 0xf6, 0x01], 3),
    ];
    for (value, second) in cases {
        let refused = refusal(&holding(value.clone()));
        let expected = DecodeError::DuplicateKey { component: minted(A), property: minted(P), at: VALUE_AT + second };
        assert_eq!(refused, expected, "{value:02x?}");
    }
    // 1 and 1.0 are two keys, and so are 0.0 and -0.0.
    assert!(read(&holding(vec![0xa2, 0x01, 0x00, 0xf9, 0x3c, 0x00, 0x00])).is_ok());
    assert!(read(&holding(vec![0xa2, 0xf9, 0x00, 0x00, 0x00, 0xf9, 0x80, 0x00, 0x00])).is_ok());
}

#[test]
fn a_map_key_that_is_an_array_or_a_map_is_refused() {
    for value in [vec![0xa1, 0x80, 0x00], vec![0xa1, 0xa0, 0x00]] {
        let expected = DecodeError::Excluded { at: VALUE_AT + 1, what: "a map key that is an array or a map" };
        assert_eq!(refusal(&holding(value.clone())), expected, "{value:02x?}");
    }
}

/// A value of `depth` levels, each an array that holds the next.
fn nested(depth: usize) -> Vec<u8> {
    let mut value = vec![0x81; depth - 1];
    value.push(0x80);
    value
}

// The envelope takes three levels down to a property of its own components, so their values
// begin at the fourth; a record's begin at the sixth.
#[test]
fn a_value_nested_deeper_than_the_limit_is_refused_and_one_at_the_limit_is_read() {
    let limits = Limits { max_depth: 10, ..Limits::default() };
    assert!(index(&holding(nested(7)), &limits, &[]).is_ok());
    assert!(matches!(index(&holding(nested(8)), &limits, &[]), Err(DecodeError::TooDeep { limit: 10, .. })));
    assert!(index(&holding_in_a_record(nested(5)), &limits, &[]).is_ok());
    assert!(matches!(index(&holding_in_a_record(nested(6)), &limits, &[]), Err(DecodeError::TooDeep { limit: 10, .. })));
    // A sender cannot exhaust the stack: the limit stops the walk.
    assert!(matches!(read(&holding(nested(100_000))), Err(DecodeError::TooDeep { limit: 32, .. })));
}

// The envelope's own items count: in `holding`, its array, the empty list, the components' map,
// A's id, its properties' map and P's id come before the value, and the records' array after it;
// in `holding_in_a_record`, the record's array, its null and its components' map come too.
#[test]
fn an_envelope_of_more_items_than_the_limit_is_refused_and_one_at_the_limit_is_read() {
    let value = |len: usize| [head(4, len as u64), vec![0x00; len]].concat();
    let limits = Limits { max_items: 6 + 11 + 1, ..Limits::default() };
    assert!(index(&holding(value(10)), &limits, &[]).is_ok());
    let over = holding(value(11));
    assert_eq!(index(&over, &limits, &[]).unwrap_err(), DecodeError::TooManyItems { at: over.len() - 1, limit: 18 });
    let limits = Limits { max_items: 10 + 11, ..Limits::default() };
    assert!(index(&holding_in_a_record(value(10)), &limits, &[]).is_ok());
    assert!(matches!(index(&holding_in_a_record(value(11)), &limits, &[]), Err(DecodeError::TooManyItems { limit: 21, .. })));
}

// A declared length or count is never trusted beyond the bytes that remain.
#[test]
fn a_length_or_count_beyond_what_remains_is_refused_without_setting_memory_aside() {
    for value in [head(2, u64::MAX), head(3, 1 << 40), head(4, u64::MAX), head(5, u64::MAX >> 1)] {
        let refused = refusal(&holding_in_a_record(value.clone()));
        assert!(matches!(refused, DecodeError::Truncated { .. }), "{value:02x?}: {refused}");
    }
    for bytes in [
        [&[0x83][..], &head(4, u64::MAX)].concat(),
        [&[0x83, 0x80][..], &head(5, u64::MAX)].concat(),
        [&[0x83, 0x80, 0xa0][..], &head(4, u64::MAX)].concat(),
    ] {
        assert!(matches!(refusal(&bytes), DecodeError::Truncated { .. }), "{bytes:02x?}");
    }
}

#[test]
fn a_must_understand_component_is_refused_unless_the_receiver_understands_it_naming_every_one() {
    let bytes = envelope(&[B, A], components(&[(A, vec![]), (B, vec![])]), &[]);
    let not_understood = |understood: &[MintedId]| index(&bytes, &Limits::default(), understood).unwrap_err();
    assert_eq!(not_understood(&[]), DecodeError::NotUnderstood { components: vec![minted(A), minted(B)] });
    assert_eq!(not_understood(&[minted(B)]), DecodeError::NotUnderstood { components: vec![minted(A)] });
    assert!(index(&bytes, &Limits::default(), &[minted(A), minted(B)]).is_ok());
    // A component held only in a record is held.
    let in_a_record = envelope(&[A], components(&[]), &[record(None, components(&[(A, vec![])]))]);
    assert!(index(&in_a_record, &Limits::default(), &[minted(A)]).is_ok());
}

// RFC 8949's deterministic encoding is how an envelope is written, not a rule it is read by.
#[test]
fn an_envelope_in_another_order_and_in_longer_forms_reads_as_the_same_components() {
    let mut record = RecordBuilder::new(None);
    record.add(&formats()).unwrap().add(&pointer()).unwrap();
    let mut envelope = EnvelopeBuilder::new();
    envelope.add(&label()).unwrap().add_must_understand(&size()).unwrap().add_record(record).unwrap();
    let canonical = envelope.encode();

    // The same components, last first, each with its properties last first, and every head in its
    // longest form.
    let parts = index(&canonical, &Limits::default(), UNDERSTOOD).unwrap();
    let rewritten = |out: &mut Vec<u8>, components: &[ComponentEntry]| {
        out.extend(long(5, components.len() as u64));
        for component in components.iter().rev() {
            out.extend([long(2, 32), component.id.as_bytes().to_vec(), long(5, component.properties.len() as u64)].concat());
            for property in parts.properties[component.properties.clone()].iter().rev() {
                out.extend([long(2, 32), property.id.as_bytes().to_vec(), canonical[property.value.clone()].to_vec()].concat());
            }
        }
    };
    let mut other = [long(4, 3), long(4, 1), long(2, 32), Size::ID.as_bytes().to_vec()].concat();
    rewritten(&mut other, &parts.components[parts.own.clone()]);
    other.extend([long(4, 1), long(4, 2), vec![0xf6]].concat());
    rewritten(&mut other, &parts.components[parts.records[0].components.clone()]);
    assert_ne!(other, canonical);

    let read = |bytes: &[u8]| {
        let envelope = ReceivedEnvelope::decode(bytes, &Limits::default(), UNDERSTOOD).unwrap();
        let own: (Label, Size) = envelope.components().read().unwrap();
        let records: Vec<(Formats, Pointer)> = envelope.records().map(|record| record.components().read().unwrap()).collect();
        (envelope.must_understand().to_vec(), own, records)
    };
    assert_eq!(read(&other), read(&canonical));
    assert_eq!(read(&canonical), (vec![Size::ID], (label(), size()), vec![(formats(), pointer())]));
}
