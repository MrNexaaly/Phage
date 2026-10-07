//! HPACK header compression for HTTP/2 (RFC 7541).
//!
//! - [`huffman`]: the static Huffman code (shared with QPACK).
//! - [`table`]: the static table and the size-bounded dynamic table.
//! - [`Decoder`] / [`Encoder`]: header blocks to fields and back.
//! - This module: the integer and string-literal primitives (sections 5.1
//!   and 5.2), which QPACK reuses with other prefix sizes.
//!
//! Tables are generated from the RFC text by `tools/protocol_tables.py`.

pub mod huffman;
pub mod table;
#[rustfmt::skip]
mod tables;

mod decoder;
mod encoder;

pub use decoder::{Decoder, FieldList};
pub use encoder::Encoder;
pub use tables::STATIC;

/// Integers above this are refused: no field or table size is that large,
/// and it bounds the continuation bytes an attacker can send.
const MAX_INT: u64 = u32::MAX as u64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HpackError {
    /// Malformed input: a connection error (`COMPRESSION_ERROR` in HTTP/2).
    Malformed(&'static str),
    Huffman(&'static str),
    /// The decoded field list exceeds the limit; the block was still decoded
    /// to keep the dynamic table in sync, so only this stream fails (431).
    TooLarge,
}

impl std::fmt::Display for HpackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HpackError::Malformed(m) => write!(f, "malformed header block: {m}"),
            HpackError::Huffman(m) => write!(f, "bad Huffman string: {m}"),
            HpackError::TooLarge => f.write_str("header list too large"),
        }
    }
}

impl std::error::Error for HpackError {}

/// Appends `value` with an N-bit prefix; `flags` fills the bits above it.
pub fn encode_int(out: &mut Vec<u8>, flags: u8, prefix_bits: u8, mut value: u64) {
    let max = (1u64 << prefix_bits) - 1;
    if value < max {
        out.push(flags | value as u8);
        return;
    }
    out.push(flags | max as u8);
    value -= max;
    while value >= 128 {
        out.push((value % 128) as u8 | 0x80);
        value /= 128;
    }
    out.push(value as u8);
}

/// Reads an integer with an N-bit prefix from the start of `input`; returns
/// the value and the bytes it took.
pub fn decode_int(input: &[u8], prefix_bits: u8) -> Result<(u64, usize), HpackError> {
    let first = *input
        .first()
        .ok_or(HpackError::Malformed("truncated integer"))?;
    let max = (1u64 << prefix_bits) - 1;
    let mut value = u64::from(first) & max;
    if value < max {
        return Ok((value, 1));
    }
    let mut shift = 0u32;
    for (i, &byte) in input[1..].iter().enumerate() {
        value += u64::from(byte & 0x7f) << shift;
        if value > MAX_INT {
            return Err(HpackError::Malformed("integer too large"));
        }
        if byte & 0x80 == 0 {
            return Ok((value, i + 2));
        }
        shift += 7;
    }
    Err(HpackError::Malformed("truncated integer"))
}

/// Appends a string literal: the H flag at bit `prefix_bits`, the length with
/// an N-bit prefix, then the bytes, Huffman-coded when that is shorter.
pub fn encode_str(out: &mut Vec<u8>, flags: u8, prefix_bits: u8, value: &[u8]) {
    let huffman_len = huffman::encoded_len(value);
    if huffman_len < value.len() {
        encode_int(
            out,
            flags | 1 << prefix_bits,
            prefix_bits,
            huffman_len as u64,
        );
        huffman::encode(value, out);
    } else {
        encode_int(out, flags, prefix_bits, value.len() as u64);
        out.extend_from_slice(value);
    }
}

/// Reads a string literal whose H flag sits at bit `prefix_bits`, appending
/// its decoded bytes to `out`. Returns the input bytes it took.
pub fn decode_str(input: &[u8], prefix_bits: u8, out: &mut Vec<u8>) -> Result<usize, HpackError> {
    let first = *input
        .first()
        .ok_or(HpackError::Malformed("truncated string"))?;
    let huffman = first & (1 << prefix_bits) != 0;
    let (len, used) = decode_int(input, prefix_bits)?;
    let end = used
        .checked_add(len as usize)
        .filter(|&end| end <= input.len())
        .ok_or(HpackError::Malformed("string runs past the block"))?;
    let raw = &input[used..end];
    if huffman {
        huffman::decode(raw, out)?;
    } else {
        out.extend_from_slice(raw);
    }
    Ok(end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integers_match_rfc_7541_appendix_c1() {
        let mut out = Vec::new();
        encode_int(&mut out, 0, 5, 10);
        assert_eq!(out, [0b01010]);
        out.clear();
        encode_int(&mut out, 0, 5, 1337);
        assert_eq!(out, [0b11111, 0b1001_1010, 0b0000_1010]);
        out.clear();
        encode_int(&mut out, 0, 8, 42);
        assert_eq!(out, [42]);
        assert_eq!(
            decode_int(&[0b11111, 0b1001_1010, 0b0000_1010], 5),
            Ok((1337, 3))
        );
        assert_eq!(
            decode_int(&[0xe0 | 10], 5),
            Ok((10, 1)),
            "flags above the prefix ignored"
        );
    }

    #[test]
    fn integers_refuse_overflow_and_truncation() {
        assert!(decode_int(&[0x1f, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f], 5).is_err());
        assert!(decode_int(&[0x1f, 0x80], 5).is_err());
        assert!(decode_int(&[], 5).is_err());
    }

    #[test]
    fn strings_pick_the_shorter_form() {
        let mut out = Vec::new();
        encode_str(&mut out, 0, 7, b"custom-key");
        assert_eq!(out[0] & 0x80, 0x80, "Huffman is shorter for text");
        let mut decoded = Vec::new();
        assert_eq!(decode_str(&out, 7, &mut decoded), Ok(out.len()));
        assert_eq!(decoded, b"custom-key");
        let mut raw = Vec::new();
        encode_str(&mut raw, 0, 7, &[0xff, 0xfe]);
        assert_eq!(raw, [2, 0xff, 0xfe], "binary stays raw");
        assert!(decode_str(&[5, b'a'], 7, &mut decoded).is_err());
    }
}
