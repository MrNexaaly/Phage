//! Bounded recursion is verified across separate environments.
#[inline(never)] fn sum(x: u8) -> u16 { if x == 0 { 0 } else { x as u16 + sum(x - 1) } }
#[unsafe(no_mangle)] pub fn phage_target(x: u8) -> bool {
    if x > 8 { return true; }
    sum(x) == (x as u16 * (x as u16 + 1)) / 2
}
