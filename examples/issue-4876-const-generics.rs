//! Explicitly instantiated u8 and bool const generics. This protects compiler
//! handling; it does not claim automatic generic harness discovery.
fn shifted<const N: u8>(x: u8) -> u16 {
    u16::from(x) << N
}
fn pick<const ENABLED: bool>(x: u8) -> u8 {
    if ENABLED { x } else { 0 }
}

#[unsafe(no_mangle)]
pub fn phage_target(x: u8) -> bool {
    shifted::<3>(x) >> 3 == u16::from(x) && pick::<true>(x) == x && pick::<false>(x) == 0
}
