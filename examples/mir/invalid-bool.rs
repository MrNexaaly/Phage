//! Validity violation remains visible in built MIR even when its result is unused.
#![allow(unnecessary_transmutes)]
#[unsafe(no_mangle)] pub fn phage_target() -> bool {
    let _value: bool = unsafe { std::mem::transmute(2_u8) };
    true
}
