//! A negative control: x=255 reaches a checked arithmetic panic.
#[unsafe(no_mangle)]
pub fn phage_target(x: u8) -> bool {
    let next = x + 1;
    next > x
}
