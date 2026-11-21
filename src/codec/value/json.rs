//! The form of a `json` property's value: a plain value, which is `null`, `false`, `true`, a
//! number or a text, or an array of plain values, such as a list of MIME types. JSON with parts, an
//! object or an array within an array, has no form: its parts would be keyed by names or by
//! position, where a part that means something on its own is a component of its own, keyed by its
//! minted id, or a record of the envelope.

use serde_json::{Number, Value as Json};

use super::{ValueProblem, CHECKED};
use crate::codec::cbor::{self, Item, Reader};
use ankurah::ValueType;

/// A json value that has parts, an object or an array within an array, which an envelope does not
/// carry.
#[derive(Debug)]
pub(crate) struct JsonWithParts;

pub(super) fn encode(json: &Json, out: &mut Vec<u8>) -> Result<(), JsonWithParts> {
    match json {
        Json::Array(items) => {
            cbor::write_head(out, cbor::ARRAY, items.len() as u64);
            items.iter().try_for_each(|item| encode_plain(item, out))
        }
        plain => encode_plain(plain, out),
    }
}

fn encode_plain(json: &Json, out: &mut Vec<u8>) -> Result<(), JsonWithParts> {
    match json {
        Json::Null => out.push(cbor::NULL),
        Json::Bool(truth) => out.push(if *truth { cbor::TRUE } else { cbor::FALSE }),
        Json::Number(number) => {
            if let Some(unsigned) = number.as_u64() {
                cbor::write_head(out, cbor::UNSIGNED, unsigned)
            } else if let Some(signed) = number.as_i64() {
                cbor::write_integer(out, signed)
            } else {
                cbor::write_float(out, number.as_f64().expect("a JSON number that is no integer is a float"))
            }
        }
        Json::String(text) => cbor::write_text(out, text),
        Json::Array(_) | Json::Object(_) => return Err(JsonWithParts),
    }
    Ok(())
}

/// Reads the json value that begins with `first`, the rest of which `reader` holds.
pub(super) fn decode(first: Item<'_>, reader: &mut Reader<'_>) -> Result<Json, ValueProblem> {
    let Item::Array(len) = first else { return decode_plain(first) };
    // The structure's check counted every element, so the length was not taken on trust.
    let mut items = Vec::with_capacity(len as usize);
    for _ in 0..len {
        items.push(decode_plain(reader.item().expect(CHECKED))?);
    }
    Ok(Json::Array(items))
}

fn decode_plain(item: Item<'_>) -> Result<Json, ValueProblem> {
    let unexpected = |found| ValueProblem::UnexpectedType { expected: ValueType::Json, found };
    Ok(match item {
        Item::Null => Json::Null,
        Item::Bool(truth) => Json::Bool(truth),
        Item::Integer(number) => match (u64::try_from(number), i64::try_from(number)) {
            (Ok(unsigned), _) => Json::from(unsigned),
            (_, Ok(signed)) => Json::from(signed),
            _ => return Err(ValueProblem::OutOfRange { expected: ValueType::Json }),
        },
        Item::Float(number) => Json::Number(Number::from_f64(number).ok_or(unexpected("a float that is not finite"))?),
        Item::Text(text) => Json::String(text.to_owned()),
        Item::Array(_) => return Err(unexpected("an array within an array")),
        other => return Err(unexpected(other.kind())),
    })
}
