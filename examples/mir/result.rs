//! Result branches and custom enum discriminants use logical Rust tags.
#![allow(dead_code)]
#[repr(u8)] enum Mode { Low = 7, High = 9 }
#[inline(never)] fn result(x: u8) -> Result<u8, u8> { if x < 10 { Ok(x) } else { Err(x) } }
#[unsafe(no_mangle)] pub fn phage_target(x: u8) -> bool {
    let actual = match result(x) { Ok(value) => value, Err(value) => value };
    let mode = if x < 10 { Mode::Low } else { Mode::High };
    actual == x && (mode as u8 == if x < 10 { 7 } else { 9 })
}
