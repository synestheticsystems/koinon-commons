//! Each item of an envelope, written and read as RFC 8949 writes it (its appendix A gives the
//! encodings), and each item no envelope holds, refused where it begins.

use super::*;

fn written(write: impl FnOnce(&mut Vec<u8>)) -> Vec<u8> {
    let mut out = Vec::new();
    write(&mut out);
    out
}

fn read(bytes: &[u8]) -> Result<Item<'_>, DecodeError> {
    Reader::new(bytes).item()
}

fn read_float(bytes: &[u8]) -> f64 {
    match read(bytes).unwrap() {
        Item::Float(number) => number,
        other => panic!("{bytes:02x?} reads as {other:?}"),
    }
}

#[test]
fn a_head_takes_the_shortest_form_that_holds_its_argument() {
    let cases: [(u64, &[u8]); 10] = [
        (0, &[0x00]),
        (23, &[0x17]),
        (24, &[0x18, 0x18]),
        (0xff, &[0x18, 0xff]),
        (0x100, &[0x19, 0x01, 0x00]),
        (0xffff, &[0x19, 0xff, 0xff]),
        (0x1_0000, &[0x1a, 0x00, 0x01, 0x00, 0x00]),
        (0xffff_ffff, &[0x1a, 0xff, 0xff, 0xff, 0xff]),
        (0x1_0000_0000, &[0x1b, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00]),
        (u64::MAX, &[0x1b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]),
    ];
    for (argument, expected) in cases {
        assert_eq!(written(|out| write_head(out, UNSIGNED, argument)), expected, "{argument}");
    }
    assert_eq!(written(|out| write_head(out, MAP, 2)), [0xa2]);
}

#[test]
fn integers_and_strings_are_written_as_rfc_8949_writes_them() {
    assert_eq!(written(|out| write_integer(out, 0)), [0x00]);
    assert_eq!(written(|out| write_integer(out, 1000)), [0x19, 0x03, 0xe8]);
    assert_eq!(written(|out| write_integer(out, -1)), [0x20]);
    assert_eq!(written(|out| write_integer(out, -1000)), [0x39, 0x03, 0xe7]);
    assert_eq!(written(|out| write_integer(out, i64::MIN)), [0x3b, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
    assert_eq!(written(|out| write_text(out, "IETF")), [0x64, 0x49, 0x45, 0x54, 0x46]);
    assert_eq!(written(|out| write_text(out, "\u{fc}")), [0x62, 0xc3, 0xbc]);
    assert_eq!(written(|out| write_bytes(out, &[1, 2, 3, 4])), [0x44, 1, 2, 3, 4]);
}

#[test]
fn every_item_reads_back_as_written() {
    assert!(matches!(read(&[0x3b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]).unwrap(), Item::Integer(n) if n == -(1 << 64)));
    assert!(matches!(read(&[0x1b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]).unwrap(), Item::Integer(n) if n == u64::MAX.into()));
    assert!(matches!(read(&[0x64, 0x49, 0x45, 0x54, 0x46]).unwrap(), Item::Text("IETF")));
    assert!(matches!(read(&[0x44, 1, 2, 3, 4]).unwrap(), Item::Bytes([1, 2, 3, 4])));
    assert!(matches!(read(&[0x98, 0x19]).unwrap(), Item::Array(25)));
    assert!(matches!(read(&[0xa2]).unwrap(), Item::Map(2)));
    assert!(matches!(read(&[FALSE]).unwrap(), Item::Bool(false)));
    assert!(matches!(read(&[TRUE]).unwrap(), Item::Bool(true)));
    assert!(matches!(read(&[NULL]).unwrap(), Item::Null));
}

#[test]
fn a_float_takes_the_shortest_width_that_keeps_every_bit() {
    let cases: [(f64, &[u8]); 11] = [
        (1.5, &[0xf9, 0x3e, 0x00]),
        (-0.0, &[0xf9, 0x80, 0x00]),
        (f64::INFINITY, &[0xf9, 0x7c, 0x00]),
        (f64::NEG_INFINITY, &[0xf9, 0xfc, 0x00]),
        (65504.0, &[0xf9, 0x7b, 0xff]),
        (5.960464477539063e-8, &[0xf9, 0x00, 0x01]),
        (100000.0, &[0xfa, 0x47, 0xc3, 0x50, 0x00]),
        (3.4028234663852886e+38, &[0xfa, 0x7f, 0x7f, 0xff, 0xff]),
        (1.1, &[0xfb, 0x3f, 0xf1, 0x99, 0x99, 0x99, 0x99, 0x99, 0x9a]),
        (1.0e+300, &[0xfb, 0x7e, 0x37, 0xe4, 0x3c, 0x88, 0x00, 0x75, 0x9c]),
        // The NaN Rust makes, which RFC 8949 section 4.2.2 suggests.
        (f64::NAN, &[0xf9, 0x7e, 0x00]),
    ];
    for (number, expected) in cases {
        let bytes = written(|out| write_float(out, number));
        assert_eq!(bytes, expected, "{number}");
        assert_eq!(read_float(&bytes).to_bits(), number.to_bits(), "{number}");
    }
}

// Signalling NaNs among them, which a conversion between widths may quieten.
#[test]
fn a_nan_keeps_its_sign_and_payload_in_whatever_width_it_takes() {
    for (bits, width) in [(0x7ff4_0000_0000_0000u64, 3), (0xfff0_0000_2000_0000, 5), (0x7ff8_0000_0000_0001, 9), (0xfff0_0000_0000_0001, 9)] {
        let bytes = written(|out| write_float(out, f64::from_bits(bits)));
        assert_eq!(bytes.len(), width, "{bits:x}");
        assert_eq!(read_float(&bytes).to_bits(), bits, "{bits:x}");
    }
}

#[test]
fn a_head_or_a_float_written_longer_than_it_needs_reads_as_the_same_item() {
    assert!(matches!(read(&[0x18, 0x01]).unwrap(), Item::Integer(1)));
    assert!(matches!(read(&[0x1b, 0, 0, 0, 0, 0, 0, 0, 0x01]).unwrap(), Item::Integer(1)));
    assert!(matches!(read(&[0x5a, 0, 0, 0, 0x01, 0xaa]).unwrap(), Item::Bytes([0xaa])));
    for wide in [&[0xfa, 0x3f, 0xc0, 0x00, 0x00][..], &[0xfb, 0x3f, 0xf8, 0, 0, 0, 0, 0, 0]] {
        assert_eq!(read_float(wide), 1.5, "{wide:02x?}");
    }
}

#[test]
fn what_no_envelope_holds_is_refused_where_it_begins() {
    let cases: [(&[u8], &str); 12] = [
        (&[0x9f, 0x01, 0xff], "an item of indefinite length"),
        (&[0xbf, 0xff], "an item of indefinite length"),
        (&[0x5f, 0x41, 0x00, 0xff], "an item of indefinite length"),
        (&[0x7f, 0x61, 0x61, 0xff], "an item of indefinite length"),
        (&[0xc1, 0x1a, 0x51, 0x4b, 0x67, 0xb0], "a tag"),
        (&[0xf7], "undefined"),
        (&[0xf0], "a simple value"),
        (&[0xf8, 0x20], "a simple value"),
        (&[0xff], "a break"),
        (&[0x1c], "a malformed head"),
        (&[0x1f], "a malformed head"),
        (&[0xdf], "a malformed head"),
    ];
    for (bytes, what) in cases {
        assert_eq!(read(bytes).unwrap_err(), DecodeError::Excluded { at: 0, what }, "{bytes:02x?}");
    }
}

#[test]
fn text_that_is_not_utf8_is_refused_where_it_begins() {
    assert_eq!(read(&[0x62, 0xc3, 0x28]).unwrap_err(), DecodeError::NotUtf8 { at: 0 });
}

// The reader never sets memory aside for a length: a string's content is borrowed once it is
// there, and an array's or a map's elements are read one by one.
#[test]
fn an_item_cut_short_is_refused_where_it_begins() {
    for bytes in [
        &[][..],
        &[0x19, 0x01],
        &[0x5b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff],
        &[0x7a, 0xff, 0xff, 0xff, 0xff, b'a'],
        &[0x43, 0x01, 0x02],
        &[0xfb, 0x3f, 0xf8],
    ] {
        assert_eq!(read(bytes).unwrap_err(), DecodeError::Truncated { at: 0 }, "{bytes:02x?}");
    }
}
