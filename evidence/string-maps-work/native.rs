#[path = "/home/zero/Dev/Nexaaly/Phage/results/1790874971847005284-3029559/sources/home/zero/Dev/Nexaaly/Phage/examples/string-map.rs"]
mod property;
fn main() {
for key in 0..=255u8 { for value in 0..=255u8 { assert!(property::phage_target(key,value)); assert!(!property::string_map_bad(key,value)); } }
println!("65536 positive and 65536 negative native inputs checked");
}
