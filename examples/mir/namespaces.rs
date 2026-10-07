#![allow(dead_code)]
fn helper(x:u8)->u8{x}
mod math {
 pub struct Pair(pub u8);
 fn helper(_x:u8)->u8{0}
 impl Pair { #[inline(never)]pub fn get(&self)->u8 {helper(self.0)} }
}
#[unsafe(no_mangle)]pub fn phage_target(x:u8)->bool {let p=math::Pair(x);p.get()==x}
