//! Structs, references, symbolic arrays, enums and a generic owned call.
#![allow(dead_code)]
struct Pair { left: u8, right: u8 }
enum State { Idle, Ready(u8) }
#[inline(never)] fn widened(x: u8) -> u16 { (x as u16) + 1 }
#[inline(never)] fn identity<T>(x: T) -> T { x }
#[unsafe(no_mangle)]
pub fn phage_target(x: u8) -> bool {
    let mut pair = Pair { right: 2, left: x };
    let reference = &mut pair;
    reference.left = x;
    let array = [x, 2, 3];
    let selected = array[(x % 3) as usize];
    let option = Some(x);
    let decoded = match option { Some(value) => value, None => 0 };
    let state = if x == 0 { State::Idle } else { State::Ready(x) };
    let state_value = match state { State::Idle => 0, State::Ready(value) => value };
    widened(x) > x as u16 && identity(decoded) == x && state_value == x
        && selected == array[(x % 3) as usize] && pair.right == 2
}
