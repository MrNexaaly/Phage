//! Kani controls call the identical production functions in check.rs.
#[path="check.rs"] mod property;
#[kani::proof]
fn mutable() {
    let mut bytes: [u8;4] = kani::any();
    assert!(property::mutable(&mut bytes, kani::any()));
}
#[kani::proof]
fn slice() {
    let bytes: [u8;16] = kani::any();
    let length: usize = kani::any();
    kani::assume(length<=16);
    assert!(property::slice(&bytes[..length],kani::any()));
}
#[kani::proof]
fn bad_value() {
    let bytes: [u8;4] = kani::any();
    assert!(property::bad_value(&bytes));
}
#[kani::proof]
fn bad_slice() {
    let bytes: [u8;16] = kani::any();
    let length: usize = kani::any();
    kani::assume(length<=16);
    assert!(property::bad_slice(&bytes[..length]));
}
