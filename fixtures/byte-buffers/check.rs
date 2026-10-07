//! Byte-buffer ABI controls, checked as retained LLVM on each toolchain.
#[unsafe(no_mangle)]
pub fn phage_target(bytes: &[u8; 4096], index: u16) -> bool {
    bytes.get(usize::from(index)).map_or(true, |b| u16::from(*b) < 256)
}
#[unsafe(no_mangle)]
pub fn slice(bytes: &[u8], index: usize) -> bool {
    bytes.get(index).map_or(true, |b| u16::from(*b) < 256)
}
#[unsafe(no_mangle)]
pub fn mutable(bytes: &mut [u8; 4], value: u8) -> bool {
    bytes[1] = value;
    bytes[1] == value
}
#[unsafe(no_mangle)]
pub fn mutable_slice(bytes: &mut [u8], value: u8) -> bool {
    if !bytes.is_empty() { bytes[0] = value; bytes[0] == value } else { true }
}
#[unsafe(no_mangle)]
pub fn bad_value(bytes: &[u8;4]) -> bool { bytes[0] == 7 }
#[unsafe(no_mangle)]
pub fn bad_slice(bytes: &[u8]) -> bool { bytes[bytes.len()] == 7 }
#[unsafe(no_mangle)]
pub fn unsupported(pointer: *const u8) -> bool { pointer.is_null() }
#[inline(never)]
fn decode(bytes: &[u8]) -> Option<u16> {
    Some(u16::from_le_bytes([*bytes.first()?, *bytes.get(1)?]))
}
#[unsafe(no_mangle)]
pub fn parser(bytes: &[u8;16], length: u8) -> bool {
    let Some(input) = bytes.get(..usize::from(length)) else {return true;};
    match decode(input) {
        Some(value) => input.len() >= 2 && value == u16::from(input[0]) + (u16::from(input[1]) << 8),
        None => input.len() < 2,
    }
}
#[unsafe(no_mangle)]
pub fn bad_parser(bytes: &[u8;16], length: u8) -> bool {
    let Some(input) = bytes.get(..usize::from(length)) else {return true;};
    match decode(input) {
        Some(value) => value == u16::from(input[0]) + (u16::from(input[1]) << 7),
        None => true,
    }
}
