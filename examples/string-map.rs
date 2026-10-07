//! String-keyed HashMap lookup, with a deliberate bad result.
use std::collections::HashMap;

pub fn property(key: u8, value: u8, bad: bool) -> bool {
    let text = if key & 1 == 0 { "alpha" } else { "bravo" };
    let mut map = HashMap::<String, u32>::new();
    map.insert(text.to_owned(), u32::from(value));
    let found = map.get(text).copied();
    let expected = u32::from(value) + u32::from(bad);
    found == Some(expected) && map.len() == 1
}

#[unsafe(no_mangle)]
pub fn phage_target(key: u8, value: u8) -> bool {
    property(key, value, false)
}

#[unsafe(no_mangle)]
pub fn string_map_bad(key: u8, value: u8) -> bool {
    property(key, value, true)
}
