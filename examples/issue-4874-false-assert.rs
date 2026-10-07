//! Negative control: strict-warning assertion forms must still detect a
//! real failing assertion at x=255, rather than merely compile successfully.
#![deny(warnings)]
const _: () = assert!(true);

#[unsafe(no_mangle)]
pub fn phage_target(x: u8) -> bool {
    let () = assert!(x != 255);
    true
}
