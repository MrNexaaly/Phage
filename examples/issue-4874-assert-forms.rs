//! Regression inspired by Kani #4874: standard assertions under strict
//! warnings, including constant and expression positions. No macro rewrite.
#![deny(warnings)]
const _: () = assert!(true);
const _: () = const { assert!(2 + 2 == 4) };

#[unsafe(no_mangle)]
pub fn phage_target(x: u8) -> bool {
    match x {
        0 => assert!(x == 0),
        _ => assert!(x > 0),
    }
    true
}
