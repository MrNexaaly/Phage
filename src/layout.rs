//! The target data layout the memory model assumes. Implicit alignments
//! (`constants::align_of` for globals without `align`, `execute::align` for
//! accesses without `align`) are fixed functions of a type's size, and
//! accesses are checked against them on the address. They are right only
//! when the module's layout gives LLVM the same ABI alignments, so any other
//! layout is refused (unknown) instead of verified against wrong alignments.

use std::collections::BTreeMap;

/// Accepts a little-endian layout whose ABI alignments for integers, floats,
/// vectors, aggregates and address-space-0 pointers are the model's.
pub fn check(text: &str) -> Result<(), String> {
    let layout = text
        .lines()
        .find_map(|l| l.strip_prefix("target datalayout = \""))
        .and_then(|l| l.strip_suffix('"'))
        .ok_or("prototype requires a target data layout")?;
    let bad = |spec: &str| format!("unsupported data layout component {spec}");
    // ABI alignments in bits by width: LLVM's defaults, then the layout's.
    let mut ints = BTreeMap::from([(1, 8), (8, 8), (16, 16), (32, 32), (64, 32)]);
    let mut floats = BTreeMap::new();
    let mut little = false;
    for spec in layout.split('-') {
        let fields: Vec<&str> = spec.split(':').collect();
        let head = fields[0];
        let numbers = || {
            fields[1..]
                .iter()
                .map(|n| n.parse::<u64>().map_err(|_| bad(spec)))
                .collect::<Result<Vec<u64>, String>>()
        };
        let abi = || numbers()?.first().copied().ok_or_else(|| bad(spec));
        match head.chars().next() {
            Some('e') if head == "e" => little = true,
            // Mangling, native widths, non-integral spaces, stack and code alignment.
            Some('m' | 'n' | 'S' | 'F') => {}
            Some(kind @ ('i' | 'f' | 'v')) => {
                let width: u64 = head[1..].parse().map_err(|_| bad(spec))?;
                let abi = abi()?;
                match kind {
                    'i' => {
                        ints.insert(width, abi);
                    }
                    'f' => {
                        floats.insert(width, abi);
                    }
                    // A vector access defaults to its size, a power of two.
                    _ if abi != width.next_power_of_two() => return Err(bad(spec)),
                    _ => {}
                }
            }
            // Address space 0: 64-bit pointers aligned to 8 with 64-bit GEP
            // indices. Other spaces are refused where they are used.
            Some('p') if head[1..].parse::<u64>().unwrap_or(0) == 0 => {
                let numbers = numbers()?;
                if numbers.first() != Some(&64)
                    || numbers.get(1).is_some_and(|a| *a != 64)
                    || numbers.get(3).is_some_and(|i| *i != 64)
                {
                    return Err(bad(spec));
                }
            }
            Some('p') => {}
            // Aggregates align to their widest field: no larger minimum.
            Some('a') if abi()? <= 8 => {}
            _ => return Err(bad(spec)),
        }
    }
    if !little {
        return Err("prototype requires little-endian LLVM layout".into());
    }
    // Every byte-sized integer access width. LLVM: an exact entry, else the
    // next wider one, else the widest. The model: the byte size rounded up
    // to a power of two, at most 16.
    for width in (8u64..=128).step_by(8) {
        let llvm = ints
            .get(&width)
            .or_else(|| ints.range(width..).next().map(|(_, a)| a))
            .or_else(|| ints.values().next_back())
            .copied();
        if llvm != Some((width / 8).next_power_of_two().min(16) * 8) {
            return Err(format!("data layout aligns i{width} unlike the model"));
        }
    }
    // The float formats the engine supports; without an entry LLVM uses
    // the natural alignment, which is the model's.
    for width in [32u64, 64] {
        let llvm = floats.get(&width).copied().unwrap_or(width);
        if llvm != width {
            return Err(format!("data layout aligns f{width} unlike the model"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::check;

    const RUSTC: &str =
        "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128";

    fn layout(text: &str) -> Result<(), String> {
        check(&format!("target datalayout = \"{text}\"\n"))
    }

    #[test]
    fn only_layouts_with_the_models_alignments_are_accepted() {
        assert!(layout(RUSTC).is_ok());
        assert!(layout(&format!("{RUSTC}-a:0:64-v128:128-p:64:64:64")).is_ok());
        // Each changes an alignment the model fixes, or the byte order.
        for (from, to) in [
            ("i128:128", "i128:64"), // LLVM before 18
            ("-i64:64", ""),         // LLVM's default i64 ABI alignment is 4
            ("i64:64", "i64:64-i32:64"),
            ("S128", "S128-f64:32"),
            ("S128", "S128-i24:64"),          // between entries
            ("p272:64:64", "p0:64:64:64:32"), // 32-bit GEP indices
            ("S128", "S128-a:64"),
            ("S128", "S128-v128:64"),
            ("p272:64:64", "p:32:32"),
            ("S128", "S128-G1"),
            ("e-m", "E-m"),
        ] {
            assert!(layout(&RUSTC.replace(from, to)).is_err(), "{to}");
        }
        assert!(check("target triple = \"x86_64\"").is_err());
    }
}
