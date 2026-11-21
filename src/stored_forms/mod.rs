//! The forms in which several of the crate's property types store and carry their values, written
//! once so that each type keeps to its form the same way: bytes as lower-case hexadecimal digits,
//! a list of distinct items as a JSON array of their texts, and an enum as the name of a variant.

mod distinct_lists;
mod enums_by_name;
mod hex;

pub(crate) use distinct_lists::{distinct_list, first_repeat, ListedText};
pub(crate) use enums_by_name::enum_stored_by_name;
pub(crate) use hex::{from_lower_hex, lower_hex};
