//! Bounded Vec operations, Option payloads and UTF-8 String composition.
#[unsafe(no_mangle)] pub fn phage_target(x: u8) -> bool {
    let mut items = Vec::<u8>::new();
    items.push(x); items.push(2);
    let previous_len = items.len();
    let popped = items.pop().unwrap();
    let mut text = String::from("hé");
    text.push_str("llo");
    previous_len == 2 && popped == 2 && items[0] == x && text == "héllo" && text.len() == 6
}
