//! All u8 inputs are widened before addition, so the property holds.
#[unsafe(no_mangle)]
pub fn phage_target(x: u8) -> bool {
    let next = u16::from(x) + 1;
    next > u16::from(x)
}
