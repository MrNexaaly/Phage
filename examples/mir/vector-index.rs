//! Bounds failure through the standard Vec indexing API.
#[unsafe(no_mangle)] pub fn phage_target(x: u8) -> bool {
    let mut items = Vec::new(); items.push(x); items[1] == x
}
