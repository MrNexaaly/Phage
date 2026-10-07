//! Same input and functional scope as Nexagate's Kani varint read proof:
//! arbitrary bytes and input lengths 0..8, truncation, size and value bounds.
#![allow(dead_code)]
#[path = "../../Nexagate/crates/gate-quic/src/varint.rs"]
mod varint;

#[unsafe(no_mangle)]
pub fn phage_target(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8, g: u8, h: u8, len: u8) -> bool {
    if len > 8 {
        return true;
    }
    let data = [a, b, c, d, e, f, g, h];
    let result = varint::read(&data[..usize::from(len)]);
    if len == 0 {
        return result.is_none();
    }
    let required = 1usize << (a >> 6);
    match result {
        Some((value, used)) => used == required && used <= usize::from(len) && value <= varint::MAX,
        None => usize::from(len) < required,
    }
}
