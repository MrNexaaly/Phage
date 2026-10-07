//! Integer std methods lower to LLVM intrinsics (ctlz, cttz, ctpop, bswap,
//! bitreverse, fshl/fshr, saturating ops, min/max, abs, three-way compare).
//! Each is checked against a straight-line reference over all inputs.
use std::cmp::Ordering;

fn bit(x: u8, i: u32) -> u32 { u32::from((x >> i) & 1) }
fn ones(x: u8) -> u32 {
    bit(x, 0) + bit(x, 1) + bit(x, 2) + bit(x, 3) + bit(x, 4) + bit(x, 5) + bit(x, 6) + bit(x, 7)
}
fn leading(x: u8) -> u32 {
    if x >= 128 { 0 } else if x >= 64 { 1 } else if x >= 32 { 2 } else if x >= 16 { 3 }
    else if x >= 8 { 4 } else if x >= 4 { 5 } else if x >= 2 { 6 } else if x == 1 { 7 } else { 8 }
}
fn trailing(x: u8) -> u32 { if x == 0 { 8 } else { leading(reverse(x)) } }
fn reverse(x: u8) -> u8 {
    let b = |i: u32| ((x >> i) & 1) << (7 - i);
    b(0) | b(1) | b(2) | b(3) | b(4) | b(5) | b(6) | b(7)
}
fn widen_add(x: i8, y: i8) -> i8 { (i16::from(x) + i16::from(y)).clamp(-128, 127) as i8 }
fn order(x: i8, y: i8) -> i8 { if x < y { -1 } else if x == y { 0 } else { 1 } }

#[unsafe(no_mangle)]
pub fn phage_target(x: u8, y: u8, k: u8) -> bool {
    let (sx, sy) = (x as i8, y as i8);
    let k = u32::from(k % 8);
    x.count_ones() == ones(x)
        && x.leading_zeros() == leading(x)
        && x.trailing_zeros() == trailing(x)
        && x.reverse_bits() == reverse(x)
        && u16::from_be_bytes([x, y]).swap_bytes() == u16::from_le_bytes([x, y])
        && x.rotate_left(k) == ((x << k) | x.checked_shr(8 - k).unwrap_or(0))
        && x.saturating_add(y) == (u16::from(x) + u16::from(y)).min(255) as u8
        && x.saturating_sub(y) == if x > y { x - y } else { 0 }
        && sx.saturating_add(sy) == widen_add(sx, sy)
        && x.min(y) == if x < y { x } else { y }
        && sx.max(sy) == if sx > sy { sx } else { sy }
        && sx.unsigned_abs() == if sx < 0 { (-i16::from(sx)) as u8 } else { sx as u8 }
        && (match sx.cmp(&sy) { Ordering::Less => -1, Ordering::Equal => 0, Ordering::Greater => 1 }) == order(sx, sy)
}
