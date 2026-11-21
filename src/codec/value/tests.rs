//! The form of each value type, and the items that do not read as a property's value.

use serde_json::json;

use super::*;

fn encoded(value: &Value) -> Vec<u8> {
    let mut out = Vec::new();
    encode(value, &mut out).unwrap();
    out
}

fn refused(item: &[u8], value_type: ValueType) -> ValueProblem {
    decode(item, value_type).unwrap_err()
}

#[test]
fn each_value_type_has_its_form() {
    let mut entity = vec![0x58, 0x20];
    entity.extend([7; 32]);
    let cases: [(Value, &[u8]); 11] = [
        (Value::String("IETF".into()), &[0x64, 0x49, 0x45, 0x54, 0x46]),
        (Value::I16(-1), &[0x20]),
        (Value::I32(1000), &[0x19, 0x03, 0xe8]),
        (Value::I64(-1000), &[0x39, 0x03, 0xe7]),
        (Value::F64(1.5), &[0xf9, 0x3e, 0x00]),
        // An integral float stays a float.
        (Value::F64(4.0), &[0xf9, 0x44, 0x00]),
        (Value::Bool(true), &[0xf5]),
        (Value::EntityId(EntityId::from_bytes([7; 32])), &entity),
        (Value::Binary(vec![1, 2, 3, 4]), &[0x44, 1, 2, 3, 4]),
        (Value::Object(vec![]), &[0x40]),
        (Value::Json(json!(["a", 1, -1, 1.5, true, null])), &[0x86, 0x61, b'a', 0x01, 0x20, 0xf9, 0x3e, 0x00, 0xf5, 0xf6]),
    ];
    for (value, expected) in cases {
        assert_eq!(encoded(&value), expected, "{value:?}");
    }
}

#[test]
fn every_value_type_reads_back_as_written() {
    for value in [
        Value::String("Pack bench 3".into()),
        Value::String(String::new()),
        Value::I16(i16::MIN),
        Value::I32(i32::MAX),
        Value::I64(i64::MIN),
        Value::I64(i64::MAX),
        Value::F64(101.6),
        Value::F64(-0.0),
        Value::F64(f64::INFINITY),
        Value::Bool(false),
        Value::EntityId(EntityId::from_bytes([9; 32])),
        Value::Binary((0..=255).collect()),
        Value::Object(vec![0xff; 3]),
        Value::Json(json!(["application/pdf", "application/vnd.cups-raw"])),
        Value::Json(json!([u64::MAX, i64::MIN, 1.0, -0.0, ""])),
        Value::Json(json!("plain")),
        Value::Json(json!(null)),
    ] {
        let bytes = encoded(&value);
        assert_eq!(decode(&bytes, type_of(&value)).unwrap(), value, "{bytes:02x?}");
    }
}

#[test]
fn an_item_of_another_form_does_not_read_as_a_propertys_value() {
    let text = encoded(&Value::String("4".into()));
    assert!(matches!(refused(&text, ValueType::I64), ValueProblem::UnexpectedType { found: "a text string", .. }));
    assert!(matches!(refused(&[0xf5], ValueType::String), ValueProblem::UnexpectedType { found: "true or false", .. }));
    assert!(matches!(refused(&[0xf6], ValueType::Bool), ValueProblem::UnexpectedType { found: "null", .. }));
    let big = encoded(&Value::I64(40_000));
    assert!(matches!(refused(&big, ValueType::I16), ValueProblem::OutOfRange { expected: ValueType::I16 }));
    let short = encoded(&Value::Binary(vec![0; 31]));
    assert!(matches!(refused(&short, ValueType::EntityId), ValueProblem::NotAnEntityId { len: 31 }));
    let id = encoded(&Value::EntityId(EntityId::from_bytes([1; 32])));
    assert!(matches!(refused(&id, ValueType::String), ValueProblem::UnexpectedType { found: "a byte string", .. }));
}

// JavaScript's encoders write every integral number as an integer.
#[test]
fn an_integer_reads_as_a_float_only_when_the_float_holds_it_exactly() {
    assert_eq!(decode(&[0x04], ValueType::F64).unwrap(), Value::F64(4.0));
    for exact in [1i128 << 53, 1 << 60, -(1 << 64)] {
        let bytes = encoded_integer(exact);
        assert_eq!(decode(&bytes, ValueType::F64).unwrap(), Value::F64(exact as f64), "{exact}");
    }
    let inexact = encoded_integer((1 << 53) + 1);
    assert!(matches!(refused(&inexact, ValueType::F64), ValueProblem::OutOfRange { expected: ValueType::F64 }));
    let float = encoded(&Value::F64(4.0));
    assert!(matches!(refused(&float, ValueType::I64), ValueProblem::UnexpectedType { found: "a float", .. }));
}

/// An integer of CBOR's whole range, which `encode` gives only within i64's.
fn encoded_integer(number: i128) -> Vec<u8> {
    let mut out = Vec::new();
    match u64::try_from(number) {
        Ok(unsigned) => cbor::write_head(&mut out, cbor::UNSIGNED, unsigned),
        Err(_) => cbor::write_head(&mut out, cbor::NEGATIVE, u64::try_from(-1 - number).unwrap()),
    }
    out
}

// Its parts would be keyed by name or by position, where a part that means something alone is a
// component of its own.
#[test]
fn json_holds_a_plain_value_or_a_list_of_them_and_nothing_with_parts() {
    for json in [json!({ "name": "4x6" }), json!([["application/pdf"]]), json!([{ "": 0 }])] {
        let mut out = Vec::new();
        assert!(encode(&Value::Json(json.clone()), &mut out).is_err(), "{json}");
    }
    let cases: [(&[u8], &str); 5] = [
        (&[0xa1, 0x61, b'k', 0x01], "a map"),
        (&[0x81, 0x81, 0x01], "an array within an array"),
        (&[0x81, 0xa0], "a map"),
        (&[0x41, 0x00], "a byte string"),
        (&[0xf9, 0x7e, 0x00], "a float that is not finite"),
    ];
    for (item, found) in cases {
        let problem = refused(item, ValueType::Json);
        assert!(matches!(problem, ValueProblem::UnexpectedType { expected: ValueType::Json, found: f } if f == found), "{item:02x?}: {problem}");
    }
    let below_i64 = encoded_integer(i128::from(i64::MIN) - 1);
    assert!(matches!(refused(&below_i64, ValueType::Json), ValueProblem::OutOfRange { expected: ValueType::Json }));
}

#[test]
fn every_value_type_ankurah_names_has_a_form() {
    for name in ["string", "i16", "i32", "i64", "f64", "bool", "entityid", "object", "binary", "json"] {
        assert_eq!(Some(value_type(name)), ValueType::from_property_str(name), "{name}");
    }
}

// In a constant, as `#[derive(Minted)]` writes one for each property, the panic fails the build.
#[test]
#[should_panic(expected = "has no form in an envelope")]
fn a_value_type_without_a_form_has_none() {
    value_type("decimal");
}
