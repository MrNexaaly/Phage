//! Phage on Phage: known-bits transfer functions (src/knownbits.rs).
#![allow(dead_code)]
mod knownbits;
mod heap {
pub(crate) fn literal(expr: &str) -> Option<u128> {
    let expr = expr.trim();
    if let Some(rest) = expr.strip_prefix("(_ bv") {
        // `(_ bvN W)` is N modulo 2^W.
        let mut words = rest.split_whitespace();
        let value: u128 = words.next()?.parse().ok()?;
        let width: u32 = words.next()?.trim_end_matches(')').parse().ok()?;
        return Some(if width >= 128 {
            value
        } else {
            value & ((1u128 << width) - 1)
        });
    }
    if let Some(hex) = expr.strip_prefix("#x") {
        return u128::from_str_radix(hex, 16).ok();
    }
    if let Some(bits) = expr.strip_prefix("#b") {
        return u128::from_str_radix(bits, 2).ok();
    }
    None
}
}
mod value {
pub fn bv(value: u128, width: u32) -> String {
    format!("(_ bv{value} {width})")
}
}

#[unsafe(no_mangle)]
pub fn phage_target(op: u8, za: u8, oa: u8, la: u8, ha: u8, x: u8, zb: u8, ob: u8, lb: u8, hb: u8, y: u8) -> bool {
    knownbits::selfcheck::check(op, [za, oa, la, ha, x], [zb, ob, lb, hb, y])
}
