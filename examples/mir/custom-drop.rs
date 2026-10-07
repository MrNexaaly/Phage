//! Destructor effects must be unknown while they are not implemented.
use std::ops::Drop as Destructor;
struct Item;
impl Destructor for Item { fn drop(&mut self) { panic!("drop executed"); } }
#[unsafe(no_mangle)] pub fn phage_target() -> bool { let _item = Item; true }
