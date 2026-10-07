//! Overflow in a non-inlined callee must not be hidden by its caller.
#[inline(never)] fn increment(x: u8) -> u8 { x + 1 }
#[unsafe(no_mangle)] pub fn phage_target(x: u8) -> bool { increment(x) > x }
