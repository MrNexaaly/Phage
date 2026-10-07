//! Phage on Phage: heap::literal never panics and reads `#x` digits
//! (5 characters from '#xb(_ 1f').
#![allow(dead_code)]
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
pub fn phage_target(b0: u8, b1: u8, b2: u8, b3: u8, b4: u8) -> bool {
    const ALPHABET: [u8; 8] = [35, 120, 98, 40, 95, 32, 49, 102];
    let data = [ALPHABET[usize::from(b0 & 7)], ALPHABET[usize::from(b1 & 7)], ALPHABET[usize::from(b2 & 7)], ALPHABET[usize::from(b3 & 7)], ALPHABET[usize::from(b4 & 7)]];
    let Ok(text) = core::str::from_utf8(&data) else {
        return true;
    };
    let value = heap::literal(text);
    match text.strip_prefix("#x") {
        Some(hex) if !hex.is_empty() && hex.bytes().all(|b| b.is_ascii_hexdigit()) => {
            value == Some(hex.bytes().fold(0u128, |v, b| {
                v * 16 + u128::from((b as char).to_digit(16).unwrap_or(0))
            }))
        }
        _ => true,
    }
}
