//! Reachable recursion depth exhaustion must be unknown.
#[allow(unconditional_recursion)]
#[inline(never)] fn forever(x: u8) -> bool { forever(x) }
#[unsafe(no_mangle)] pub fn phage_target(x: u8) -> bool { forever(x) }
