//! Bytes as lower-case hexadecimal digits: the one text that a key or a hash has, so that two
//! texts name the same bytes exactly when they are equal.

const DIGITS: &[u8; 16] = b"0123456789abcdef";

/// `bytes` as two lower-case hexadecimal digits each.
pub(crate) fn lower_hex(bytes: &[u8]) -> String {
    bytes.iter().flat_map(|byte| [DIGITS[usize::from(byte >> 4)], DIGITS[usize::from(byte & 0xf)]]).map(char::from).collect()
}

/// The `N` bytes that `text` writes as `2 * N` lower-case hexadecimal digits, and None for a text
/// of any other shape, digits in upper case among them.
pub(crate) fn from_lower_hex<const N: usize>(text: &str) -> Option<[u8; N]> {
    let digits = text.as_bytes();
    if digits.len() != 2 * N {
        return None;
    }
    let value = |digit: u8| DIGITS.iter().position(|&known| known == digit).map(|value| value as u8);
    let mut bytes = [0; N];
    for (byte, [high, low]) in bytes.iter_mut().zip(digits.as_chunks::<2>().0) {
        *byte = value(*high)? << 4 | value(*low)?;
    }
    Some(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_read_back_from_their_text() {
        let bytes: [u8; 4] = [0x00, 0x9f, 0xa0, 0xff];
        assert_eq!(lower_hex(&bytes), "009fa0ff");
        assert_eq!(from_lower_hex::<4>("009fa0ff"), Some(bytes));
    }

    #[test]
    fn a_text_of_another_length_or_with_another_digit_names_no_bytes() {
        for refused in ["009fa0f", "009fa0ff00", "009FA0FF", "009fa0fg", "+09fa0ff"] {
            assert_eq!(from_lower_hex::<4>(refused), None, "{refused}");
        }
    }
}
