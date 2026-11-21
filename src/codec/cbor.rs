//! The CBOR (RFC 8949) an envelope is made of, read and written one item at a time: the head that
//! begins every item, the content of a string, and a float in the width that keeps every bit.
//!
//! An envelope holds integers, floats, byte strings, text strings, arrays, maps, `false`, `true`
//! and `null`, each of definite length. It holds no tag, no `undefined`, no other simple value and
//! no item of indefinite length, so that no node refuses for its form an item another accepts.
//! Reading accepts a head or a float written longer than it needs to be; writing gives the
//! shortest.

use super::DecodeError;

#[cfg(test)]
mod tests;

/// The major types of RFC 8949 section 3.1 that an envelope holds.
pub(crate) const UNSIGNED: u8 = 0;
pub(crate) const NEGATIVE: u8 = 1;
pub(crate) const BYTES: u8 = 2;
pub(crate) const TEXT: u8 = 3;
pub(crate) const ARRAY: u8 = 4;
pub(crate) const MAP: u8 = 5;
const TAG: u8 = 6;
const SIMPLE: u8 = 7;

/// The items of major type 7 that an envelope holds whole in their one byte.
pub(crate) const FALSE: u8 = 0xf4;
pub(crate) const TRUE: u8 = 0xf5;
pub(crate) const NULL: u8 = 0xf6;

/// One item as an envelope may hold it: a scalar whole, or the head of an array or a map, whose
/// elements follow it.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Item<'b> {
    /// An integer of major type 0, or of major type 1, which RFC 8949 reads as `-1 - n`.
    Integer(i128),
    /// A float of any width, with every bit it was written with.
    Float(f64),
    Bytes(&'b [u8]),
    Text(&'b str),
    /// The head of an array of this many elements.
    Array(u64),
    /// The head of a map of this many pairs.
    Map(u64),
    Bool(bool),
    Null,
}

impl Item<'_> {
    /// What a person calls an item of this kind, in an error that says what was found.
    pub(crate) fn kind(&self) -> &'static str {
        match self {
            Item::Integer(_) => "an integer",
            Item::Float(_) => "a float",
            Item::Bytes(_) => "a byte string",
            Item::Text(_) => "a text string",
            Item::Array(_) => "an array",
            Item::Map(_) => "a map",
            Item::Bool(_) => "true or false",
            Item::Null => "null",
        }
    }
}

/// Reads one item after another from received bytes. It never trusts a length beyond the bytes
/// that remain, and borrows every string rather than copying it.
pub(crate) struct Reader<'b> {
    bytes: &'b [u8],
    position: usize,
}

impl<'b> Reader<'b> {
    pub(crate) fn new(bytes: &'b [u8]) -> Self {
        Reader { bytes, position: 0 }
    }

    /// Where the next item begins.
    pub(crate) fn position(&self) -> usize {
        self.position
    }

    /// Reads the next item, refusing one an envelope does not hold.
    pub(crate) fn item(&mut self) -> Result<Item<'b>, DecodeError> {
        let at = self.position;
        let &initial = self.bytes.get(at).ok_or(DecodeError::Truncated { at })?;
        let (major, info) = (initial >> 5, initial & 0x1f);
        let width = match info {
            0..=23 => 0,
            24 => 1,
            25 => 2,
            26 => 4,
            27 => 8,
            31 if (BYTES..=MAP).contains(&major) => return Err(DecodeError::Excluded { at, what: "an item of indefinite length" }),
            31 if major == SIMPLE => return Err(DecodeError::Excluded { at, what: "a break" }),
            _ => return Err(DecodeError::Excluded { at, what: "a malformed head" }),
        };
        let following = self.bytes.get(at + 1..at + 1 + width).ok_or(DecodeError::Truncated { at })?;
        let argument = if width == 0 { u64::from(info) } else { following.iter().fold(0, |argument, &byte| argument << 8 | u64::from(byte)) };
        self.position = at + 1 + width;
        Ok(match major {
            UNSIGNED => Item::Integer(i128::from(argument)),
            NEGATIVE => Item::Integer(-1 - i128::from(argument)),
            BYTES => Item::Bytes(self.content(at, argument)?),
            TEXT => Item::Text(std::str::from_utf8(self.content(at, argument)?).map_err(|_| DecodeError::NotUtf8 { at })?),
            ARRAY => Item::Array(argument),
            MAP => Item::Map(argument),
            TAG => return Err(DecodeError::Excluded { at, what: "a tag" }),
            _ => match info {
                20 => Item::Bool(false),
                21 => Item::Bool(true),
                22 => Item::Null,
                23 => return Err(DecodeError::Excluded { at, what: "undefined" }),
                25 => Item::Float(widen_half(argument as u16)),
                26 => Item::Float(widen_single(argument as u32)),
                27 => Item::Float(f64::from_bits(argument)),
                _ => return Err(DecodeError::Excluded { at, what: "a simple value" }),
            },
        })
    }

    /// The content of the string whose head begins at `at`: the `len` bytes after its head.
    fn content(&mut self, at: usize, len: u64) -> Result<&'b [u8], DecodeError> {
        let start = self.position;
        if len > (self.bytes.len() - start) as u64 {
            return Err(DecodeError::Truncated { at });
        }
        self.position = start + len as usize;
        Ok(&self.bytes[start..self.position])
    }
}

/// A half-width float as the double that holds it, a NaN with its sign and payload, which a
/// conversion may change.
fn widen_half(bits: u16) -> f64 {
    if bits & 0x7c00 == 0x7c00 && bits & 0x03ff != 0 {
        return f64::from_bits(u64::from(bits >> 15) << 63 | 0x7ff << 52 | u64::from(bits & 0x03ff) << 42);
    }
    half::f16::from_bits(bits).to_f64()
}

/// A single-width float as the double that holds it, a NaN with its sign and payload.
fn widen_single(bits: u32) -> f64 {
    if bits & 0x7f80_0000 == 0x7f80_0000 && bits & 0x007f_ffff != 0 {
        return f64::from_bits(u64::from(bits >> 31) << 63 | 0x7ff << 52 | u64::from(bits & 0x007f_ffff) << 29);
    }
    f64::from(f32::from_bits(bits))
}

/// Appends the head of an item of major type `major` whose argument is `argument`, in the shortest
/// form that holds the argument.
pub(crate) fn write_head(out: &mut Vec<u8>, major: u8, argument: u64) {
    let major = major << 5;
    match argument {
        0..=23 => out.push(major | argument as u8),
        24..=0xff => out.extend([major | 24, argument as u8]),
        0x100..=0xffff => {
            out.push(major | 25);
            out.extend((argument as u16).to_be_bytes());
        }
        0x1_0000..=0xffff_ffff => {
            out.push(major | 26);
            out.extend((argument as u32).to_be_bytes());
        }
        _ => {
            out.push(major | 27);
            out.extend(argument.to_be_bytes());
        }
    }
}

pub(crate) fn write_bytes(out: &mut Vec<u8>, bytes: &[u8]) {
    write_head(out, BYTES, bytes.len() as u64);
    out.extend_from_slice(bytes);
}

pub(crate) fn write_text(out: &mut Vec<u8>, text: &str) {
    write_head(out, TEXT, text.len() as u64);
    out.extend_from_slice(text.as_bytes());
}

pub(crate) fn write_integer(out: &mut Vec<u8>, number: i64) {
    match u64::try_from(number) {
        Ok(unsigned) => write_head(out, UNSIGNED, unsigned),
        // -1 - n, which for a negative number is its bits inverted.
        Err(_) => write_head(out, NEGATIVE, !number as u64),
    }
}

/// Appends `number` as a float of the shortest width that keeps every bit of it, a NaN's sign and
/// payload among them (RFC 8949 section 4.2.1). The NaN Rust makes is the `f97e00` that section
/// 4.2.2 suggests. An integral float stays a float.
pub(crate) fn write_float(out: &mut Vec<u8>, number: f64) {
    let bits = number.to_bits();
    if number.is_nan() {
        // A NaN narrows exactly when the payload bits that a narrower float lacks are zero.
        let (sign, payload) = (bits >> 63, bits & ((1 << 52) - 1));
        if payload & ((1 << 42) - 1) == 0 {
            out.push(0xf9);
            out.extend(((sign << 15) as u16 | 0x7c00 | (payload >> 42) as u16).to_be_bytes());
        } else if payload & ((1 << 29) - 1) == 0 {
            out.push(0xfa);
            out.extend(((sign << 31) as u32 | 0x7f80_0000 | (payload >> 29) as u32).to_be_bytes());
        } else {
            out.push(0xfb);
            out.extend(bits.to_be_bytes());
        }
        return;
    }
    let half = half::f16::from_f64(number);
    let single = number as f32;
    if half.to_f64().to_bits() == bits {
        out.push(0xf9);
        out.extend(half.to_bits().to_be_bytes());
    } else if f64::from(single).to_bits() == bits {
        out.push(0xfa);
        out.extend(single.to_bits().to_be_bytes());
    } else {
        out.push(0xfb);
        out.extend(bits.to_be_bytes());
    }
}
