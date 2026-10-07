//! Native replay of RuHealth's first HPACK counterexample.
#![allow(dead_code, unused_imports)]
#[path = "../../examples/nexagate-hpack-after.rs"]
mod after;
fn main() {
    let original = std::panic::catch_unwind(|| after::ruhealth_target(255,129,174,138,128,128,128,128,128,128,128,0,12,8));
    assert_eq!(original.ok(), Some(true), "fixed HPACK must return safely");
    println!("confirmed: fixed HPACK returned safely on the same input");
}
