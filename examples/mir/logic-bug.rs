//! A helper chooses the wrong state at x=7. Arithmetic remains safe.
#[inline(never)] fn classify(x: u8) -> bool { x > 7 }
#[unsafe(no_mangle)] pub fn phage_target(x: u8) -> bool { classify(x) == (x >= 7) }
