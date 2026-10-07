//! Symbolic HPACK integer inputs: all 12 bytes, lengths 0..12, prefixes 1..8.
#![allow(dead_code)]
#[path = "../fixtures/nexagate-before/hpack/mod.rs"]
mod hpack;
#[unsafe(no_mangle)]
pub fn phage_target(
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    g: u8,
    h: u8,
    i: u8,
    j: u8,
    k: u8,
    l: u8,
    len: u8,
    prefix: u8,
) -> bool {
    if len > 12 || prefix < 1 || prefix > 8 {
        return true;
    }
    let data = [a, b, c, d, e, f, g, h, i, j, k, l];
    match hpack::decode_int(&data[..usize::from(len)], prefix) {
        Ok((value, used)) => used > 0 && used <= usize::from(len) && value <= u64::from(u32::MAX),
        Err(_) => true,
    }
}
