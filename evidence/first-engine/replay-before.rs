//! Native replay of RuHealth's first HPACK counterexample.
#![allow(dead_code, unused_imports)]
#[path = "../../examples/nexagate-hpack-before.rs"]
mod before;
fn main() {
    let original = std::panic::catch_unwind(|| before::ruhealth_target(255,129,174,138,128,128,128,128,128,128,128,0,12,8));
    assert!(original.is_err(), "old HPACK must reproduce the shift panic");
    println!("confirmed: old HPACK panicked on the symbolic counterexample");
}
