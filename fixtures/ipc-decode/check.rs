//! Property: decoding any client/server frame payload of up to 12 bytes never panics or hits UB.
#![allow(dead_code)]
#[path = "ipc_protocol.rs"]
mod protocol;

#[unsafe(no_mangle)]
pub fn phage_target(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8, g: u8, h: u8, i: u8, j: u8, k: u8, l: u8, len: u8) -> bool {
    if len > 12 {
        return true;
    }
    let data = [a, b, c, d, e, f, g, h, i, j, k, l];
    let payload = &data[..usize::from(len)];
    let _ = protocol::decode_request(payload);
    let _ = protocol::decode_reply(payload);
    true
}
