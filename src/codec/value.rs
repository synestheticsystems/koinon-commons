//! The form a property's value takes in an envelope. The property's value type, which its kept
//! mint fixes for good, decides the form, so that a node in any language reads a property from its
//! definition alone:
//!
//! | value type        | CBOR                                                                     |
//! |-------------------|--------------------------------------------------------------------------|
//! | `string`          | a text string                                                            |
//! | `i16` `i32` `i64` | an integer in the type's range                                           |
//! | `f64`             | a float, in the shortest width that keeps every bit; an integer reads as |
//! |                   | the float when the float holds it exactly                                |
//! | `bool`            | `false` or `true`                                                        |
//! | `entityid`        | a byte string of 32 bytes                                                |
//! | `binary` `object` | a byte string                                                            |
//! | `json`            | `null`, `false`, `true`, an integer, a finite float or a text string, or |
//! |                   | an array of those                                                        |
//!
//! An integer reads as an `f64` because JavaScript's encoders write every integral number as one.
//! A value goes through its type's `Property` impl both ways, as a stored record's does, so that a
//! value the type's own rule refuses is refused in an envelope too.

mod json;
#[cfg(test)]
mod tests;

use ankurah::{property::PropertyError, EntityId, Value, ValueType};

use super::cbor::{self, Item, Reader};

pub(crate) use json::JsonWithParts;

/// Why a property's value in a received envelope does not read.
#[derive(Debug, thiserror::Error)]
pub enum ValueProblem {
    #[error("it is {found}, where {} property holds {}", named(*expected), form(*expected))]
    UnexpectedType { expected: ValueType, found: &'static str },
    #[error("it lies outside the range of {} property", named(*expected))]
    OutOfRange { expected: ValueType },
    #[error("it is a byte string of {len} bytes, where an entity id has 32")]
    NotAnEntityId { len: usize },
    /// The property type's own rule refused the value, as it would a stored record's.
    #[error("{0}")]
    Refused(Box<PropertyError>),
}

/// The value type as the catalog names it, as the property of a kept mint states it.
fn named(value_type: ValueType) -> &'static str {
    match value_type {
        ValueType::String => "a string",
        ValueType::I16 => "an i16",
        ValueType::I32 => "an i32",
        ValueType::I64 => "an i64",
        ValueType::F64 => "an f64",
        ValueType::Bool => "a bool",
        ValueType::EntityId => "an entityid",
        ValueType::Object => "an object",
        ValueType::Binary => "a binary",
        ValueType::Json => "a json",
    }
}

fn form(value_type: ValueType) -> &'static str {
    match value_type {
        ValueType::String => "a text string",
        ValueType::I16 | ValueType::I32 | ValueType::I64 => "an integer",
        ValueType::F64 => "a float or an integer",
        ValueType::Bool => "true or false",
        ValueType::EntityId | ValueType::Binary | ValueType::Object => "a byte string",
        ValueType::Json => "null, true, false, a number or a text string, or an array of those",
    }
}

/// The value type a `Property` declares, by the name the catalog gives it. A name without a form
/// in an envelope panics, which in a constant fails the build, so that no value travels before its
/// type has a form.
pub(crate) const fn value_type(name: &str) -> ValueType {
    match name.as_bytes() {
        b"string" => ValueType::String,
        b"i16" => ValueType::I16,
        b"i32" => ValueType::I32,
        b"i64" => ValueType::I64,
        b"f64" => ValueType::F64,
        b"bool" => ValueType::Bool,
        b"entityid" => ValueType::EntityId,
        b"object" => ValueType::Object,
        b"binary" => ValueType::Binary,
        b"json" => ValueType::Json,
        _ => panic!("a property's VALUE_TYPE names a value type that has no form in an envelope"),
    }
}

/// The value type of `value`'s variant.
pub(crate) fn type_of(value: &Value) -> ValueType {
    match value {
        Value::I16(_) => ValueType::I16,
        Value::I32(_) => ValueType::I32,
        Value::I64(_) => ValueType::I64,
        Value::F64(_) => ValueType::F64,
        Value::Bool(_) => ValueType::Bool,
        Value::String(_) => ValueType::String,
        Value::EntityId(_) => ValueType::EntityId,
        Value::Object(_) => ValueType::Object,
        Value::Binary(_) => ValueType::Binary,
        Value::Json(_) => ValueType::Json,
    }
}

/// Appends `value` in its form, in RFC 8949's core deterministic encoding. JSON with parts has no
/// form.
pub(crate) fn encode(value: &Value, out: &mut Vec<u8>) -> Result<(), JsonWithParts> {
    match value {
        Value::I16(number) => cbor::write_integer(out, i64::from(*number)),
        Value::I32(number) => cbor::write_integer(out, i64::from(*number)),
        Value::I64(number) => cbor::write_integer(out, *number),
        Value::F64(number) => cbor::write_float(out, *number),
        Value::Bool(truth) => out.push(if *truth { cbor::TRUE } else { cbor::FALSE }),
        Value::String(text) => cbor::write_text(out, text),
        Value::EntityId(id) => cbor::write_bytes(out, &id.to_bytes()),
        Value::Object(bytes) | Value::Binary(bytes) => cbor::write_bytes(out, bytes),
        Value::Json(json) => json::encode(json, out)?,
    }
    Ok(())
}

/// What a received envelope's structure was checked to hold, which reading a value of it relies
/// on.
pub(crate) const CHECKED: &str = "the structure of a received envelope is checked before any value of it is read";

/// Reads `item`, the bytes of one value in a received envelope, as a value of `value_type`.
pub(crate) fn decode(item: &[u8], value_type: ValueType) -> Result<Value, ValueProblem> {
    let mut reader = Reader::new(item);
    let read = reader.item().expect(CHECKED);
    let out_of_range = || ValueProblem::OutOfRange { expected: value_type };
    Ok(match (value_type, read) {
        (ValueType::String, Item::Text(text)) => Value::String(text.to_owned()),
        (ValueType::I16, Item::Integer(number)) => Value::I16(number.try_into().map_err(|_| out_of_range())?),
        (ValueType::I32, Item::Integer(number)) => Value::I32(number.try_into().map_err(|_| out_of_range())?),
        (ValueType::I64, Item::Integer(number)) => Value::I64(number.try_into().map_err(|_| out_of_range())?),
        (ValueType::F64, Item::Float(number)) => Value::F64(number),
        (ValueType::F64, Item::Integer(number)) => {
            // Every CBOR integer converts to a float in i128's range, so a round trip tells
            // whether the float holds it exactly.
            let float = number as f64;
            if float as i128 != number {
                return Err(out_of_range());
            }
            Value::F64(float)
        }
        (ValueType::Bool, Item::Bool(truth)) => Value::Bool(truth),
        (ValueType::EntityId, Item::Bytes(bytes)) => {
            Value::EntityId(EntityId::from_bytes(bytes.try_into().map_err(|_| ValueProblem::NotAnEntityId { len: bytes.len() })?))
        }
        (ValueType::Binary, Item::Bytes(bytes)) => Value::Binary(bytes.to_vec()),
        (ValueType::Object, Item::Bytes(bytes)) => Value::Object(bytes.to_vec()),
        (ValueType::Json, read) => Value::Json(json::decode(read, &mut reader)?),
        (expected, read) => return Err(ValueProblem::UnexpectedType { expected, found: read.kind() }),
    })
}
