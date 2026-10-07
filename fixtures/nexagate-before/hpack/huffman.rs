//! The static Huffman code of RFC 7541 section 5.2 and Appendix B, shared by
//! HPACK and QPACK.
//!
//! The code is canonical (codes of one length are consecutive, shorter codes
//! sort first), so decoding needs only the count of codes per length and the
//! symbols in code order: read bits until the accumulated code falls inside
//! the range of codes of the current length (the `puff.c` method). A test
//! proves the table is canonical before we rely on that.
//!
//! Decoding is strict: the padding after the last symbol must be fewer than
//! eight bits, all ones (a prefix of EOS), and EOS itself is an error.

use super::{HpackError, tables::HUFFMAN};

const EOS: u16 = 256;
const MAX_BITS: usize = 30;

/// Codes per bit length, and symbols sorted by (length, code).
const CANONICAL: ([u16; MAX_BITS + 1], [u16; 257]) = {
    let mut count = [0u16; MAX_BITS + 1];
    let mut i = 0;
    while i < 257 {
        count[HUFFMAN[i].1 as usize] += 1;
        i += 1;
    }
    let mut order = [0u16; 257];
    let mut n = 0;
    let mut len = 1;
    while len <= MAX_BITS {
        // Within one length, symbol order is code order (checked by a test).
        let mut symbol = 0;
        while symbol < 257 {
            if HUFFMAN[symbol].1 as usize == len {
                order[n] = symbol as u16;
                n += 1;
            }
            symbol += 1;
        }
        len += 1;
    }
    (count, order)
};

/// Bytes `input` takes Huffman-encoded.
pub fn encoded_len(input: &[u8]) -> usize {
    let bits: usize = input
        .iter()
        .map(|&b| usize::from(HUFFMAN[usize::from(b)].1))
        .sum();
    bits.div_ceil(8)
}

pub fn encode(input: &[u8], out: &mut Vec<u8>) {
    let mut acc: u64 = 0;
    let mut bits = 0u32;
    for &b in input {
        let (code, len) = HUFFMAN[usize::from(b)];
        acc = (acc << len) | u64::from(code);
        bits += u32::from(len);
        while bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    if bits > 0 {
        // Pad with the most significant bits of EOS: ones.
        out.push(((acc << (8 - bits)) | (0xff >> bits)) as u8);
    }
}

/// Appends the decoded bytes of `input` to `out`.
pub fn decode(input: &[u8], out: &mut Vec<u8>) -> Result<(), HpackError> {
    let (count, order) = &CANONICAL;
    let (mut code, mut first, mut index, mut len) = (0u32, 0u32, 0u32, 0usize);
    let mut all_ones = true;
    for &byte in input {
        for shift in (0..8).rev() {
            let bit = u32::from((byte >> shift) & 1);
            all_ones &= bit == 1;
            code |= bit;
            len += 1;
            let c = u32::from(count[len]);
            if code < first + c {
                let symbol = order[(index + code - first) as usize];
                if symbol == EOS {
                    return Err(HpackError::Huffman("EOS inside a string"));
                }
                out.push(symbol as u8);
                (code, first, index, len) = (0, 0, 0, 0);
                all_ones = true;
            } else {
                index += c;
                first = (first + c) << 1;
                code <<= 1;
                if len == MAX_BITS {
                    return Err(HpackError::Huffman("invalid code"));
                }
            }
        }
    }
    if len > 7 || !all_ones {
        return Err(HpackError::Huffman("invalid padding"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_is_canonical() {
        // Rebuild canonical codes from the lengths alone; they must equal the RFC's.
        let (count, order) = &CANONICAL;
        let mut next = [0u32; MAX_BITS + 2];
        let mut code = 0u32;
        for len in 1..=MAX_BITS {
            code = (code + u32::from(count[len - 1])) << 1;
            next[len] = code;
        }
        for &symbol in order.iter() {
            let (expected, len) = HUFFMAN[usize::from(symbol)];
            assert_eq!(next[usize::from(len)], expected, "symbol {symbol}");
            next[usize::from(len)] += 1;
        }
    }

    #[test]
    fn round_trips_every_byte() {
        let input: Vec<u8> = (0..=255u8)
            .chain(b"www.example.com".iter().copied())
            .collect();
        let mut encoded = Vec::new();
        encode(&input, &mut encoded);
        assert_eq!(encoded.len(), encoded_len(&input));
        let mut decoded = Vec::new();
        decode(&encoded, &mut decoded).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn rfc_example_strings() {
        // RFC 7541 C.4.1: "www.example.com" is f1e3 c2e5 f23a 6ba0 ab90 f4ff.
        let mut out = Vec::new();
        encode(b"www.example.com", &mut out);
        assert_eq!(
            out,
            [
                0xf1, 0xe3, 0xc2, 0xe5, 0xf2, 0x3a, 0x6b, 0xa0, 0xab, 0x90, 0xf4, 0xff
            ]
        );
        let mut decoded = Vec::new();
        decode(&out, &mut decoded).unwrap();
        assert_eq!(decoded, b"www.example.com");
    }

    #[test]
    fn rejects_bad_padding_and_eos() {
        let mut out = Vec::new();
        // "a" is 00011 (5 bits); padding with zeros is invalid.
        assert!(decode(&[0b0001_1000], &mut out).is_err());
        assert!(decode(&[0b0001_1111], &mut out).is_ok());
        // A whole byte of padding is too long.
        assert!(decode(&[0b0001_1111, 0xff], &mut out).is_err());
        // EOS encoded explicitly (30 ones) is an error.
        assert!(decode(&[0xff, 0xff, 0xff, 0xfc], &mut out).is_err());
    }
}
