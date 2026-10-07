//! Calling an external function must produce unknown, not a proof.
unsafe extern "C" {
    fn external_result(x: u8) -> bool;
}
#[unsafe(no_mangle)]
pub fn phage_target(x: u8) -> bool {
    // This fixture exercises unmodeled FFI; it is compiled, never executed.
    unsafe { external_result(x) }
}
