//! An incorrect length rule after pop is a logic counterexample.
#[unsafe(no_mangle)] pub fn phage_target(x: u8) -> bool {
    let mut items = Vec::new(); items.push(x); items.pop(); items.len() == 1
}
