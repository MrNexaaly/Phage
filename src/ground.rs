//! Ground evaluation of the SMT-LIB terms the engine builds: a term made
//! only of literals folds to a literal, so constants stay constants instead
//! of becoming solver symbols, and literal branch conditions and obligations
//! need no query. Semantics follow SMT-LIB's fixed-size bit-vector theory
//! exactly; anything else (symbols, arrays, division by zero) is left alone.
//! A differential test compares every operator against Z3.

/// A folded value: a Boolean or a bit-vector of `width` bits (≤ 128).
#[derive(Clone, Copy, Debug, PartialEq)]
enum Ground {
    Bool(bool),
    Bits(u128, u32),
}

/// The literal a ground `term` evaluates to, if it is ground.
pub fn fold(term: &str) -> Option<String> {
    let term = term.trim();
    if !term.starts_with('(') || term.len() > 16_384 || crate::heap::literal(term).is_some() {
        return None;
    }
    // A linear scan rejects any symbol before the recursive walk, so deep
    // array store chains are never traversed.
    let ground = term
        .split(|c: char| c == '(' || c == ')' || c.is_whitespace())
        .filter(|token| !token.is_empty())
        .all(|token| {
            token.bytes().all(|b| b.is_ascii_digit())
                || token
                    .strip_prefix("bv")
                    .is_some_and(|n| n.bytes().all(|b| b.is_ascii_digit()))
                || OPERATORS.contains(&token)
        });
    if !ground {
        return None;
    }
    let (value, rest) = eval(term, 0)?;
    if !rest.trim().is_empty() {
        return None;
    }
    Some(match value {
        Ground::Bool(b) => b.to_string(),
        Ground::Bits(v, w) => crate::value::bv(v, w),
    })
}

const OPERATORS: [&str; 34] = [
    "_",
    "true",
    "false",
    "not",
    "and",
    "or",
    "=",
    "ite",
    "bvnot",
    "bvneg",
    "concat",
    "extract",
    "zero_extend",
    "sign_extend",
    "bvadd",
    "bvsub",
    "bvmul",
    "bvand",
    "bvor",
    "bvxor",
    "bvshl",
    "bvlshr",
    "bvashr",
    "bvudiv",
    "bvurem",
    "bvult",
    "bvule",
    "bvugt",
    "bvuge",
    "bvslt",
    "bvsle",
    "bvsgt",
    "bvsge",
    "distinct",
];

fn mask(width: u32) -> u128 {
    if width == 128 {
        u128::MAX
    } else {
        (1u128 << width) - 1
    }
}
fn signed(value: u128, width: u32) -> i128 {
    let shift = 128 - width;
    ((value << shift) as i128) >> shift
}

/// Evaluates one term at the start of `text`, returning the rest.
fn eval(text: &str, depth: usize) -> Option<(Ground, &str)> {
    if depth > 256 {
        return None;
    }
    let text = text.trim_start();
    if let Some(rest) = text.strip_prefix("true") {
        return Some((Ground::Bool(true), rest));
    }
    if let Some(rest) = text.strip_prefix("false") {
        return Some((Ground::Bool(false), rest));
    }
    if let Some(rest) = text.strip_prefix("(_ bv") {
        let (value, rest) = rest.split_once(' ')?;
        let (width, rest) = rest.split_once(')')?;
        let width: u32 = width.trim().parse().ok()?;
        if width == 0 || width > 128 {
            return None;
        }
        // SMT-LIB reads `(_ bvN W)` as N modulo 2^W.
        return Some((
            Ground::Bits(value.parse::<u128>().ok()? & mask(width), width),
            rest,
        ));
    }
    let rest = text.strip_prefix('(')?;
    // Indexed operators: ((_ extract h l) x), ((_ zero_extend n) x), ...
    if let Some(indexed) = rest.strip_prefix("(_ ") {
        let (head, rest) = indexed.split_once(')')?;
        let words: Vec<&str> = head.split_whitespace().collect();
        let (x, rest) = eval(rest, depth + 1)?;
        let rest = rest.trim_start().strip_prefix(')')?;
        let Ground::Bits(v, w) = x else { return None };
        let n = |i: usize| words.get(i)?.parse::<u32>().ok();
        let out = match *words.first()? {
            "extract" => {
                let (high, low) = (n(1)?, n(2)?);
                if high < low || high >= w {
                    return None;
                }
                Ground::Bits((v >> low) & mask(high - low + 1), high - low + 1)
            }
            "zero_extend" => {
                let k = n(1)?;
                (w + k <= 128).then_some(Ground::Bits(v, w + k))?
            }
            "sign_extend" => {
                let k = n(1)?;
                if w + k > 128 {
                    return None;
                }
                Ground::Bits((signed(v, w) as u128) & mask(w + k), w + k)
            }
            _ => return None,
        };
        return Some((out, rest));
    }
    let (op, mut rest) = rest.split_once(' ')?;
    let mut args = Vec::new();
    loop {
        let trimmed = rest.trim_start();
        if let Some(after) = trimmed.strip_prefix(')') {
            rest = after;
            break;
        }
        let (value, after) = eval(trimmed, depth + 1)?;
        args.push(value);
        rest = after;
        if args.len() > 64 {
            return None;
        }
    }
    Some((apply(op, &args)?, rest))
}

fn apply(op: &str, args: &[Ground]) -> Option<Ground> {
    use Ground::{Bits, Bool};
    let bools = || -> Option<Vec<bool>> {
        args.iter()
            .map(|a| match a {
                Bool(b) => Some(*b),
                _ => None,
            })
            .collect()
    };
    match (op, args) {
        ("not", [Bool(b)]) => Some(Bool(!b)),
        ("and", _) => Some(Bool(bools()?.iter().all(|b| *b))),
        ("or", _) => Some(Bool(bools()?.iter().any(|b| *b))),
        ("=", [a, b]) => match (a, b) {
            (Bool(x), Bool(y)) => Some(Bool(x == y)),
            (Bits(x, w), Bits(y, v)) if w == v => Some(Bool(x == y)),
            _ => None,
        },
        ("ite", [Bool(c), a, b]) => match (a, b) {
            (Bool(_), Bool(_)) => Some(if *c { *a } else { *b }),
            (Bits(_, w), Bits(_, v)) if w == v => Some(if *c { *a } else { *b }),
            _ => None,
        },
        ("bvnot", [Bits(x, w)]) => Some(Bits(!x & mask(*w), *w)),
        ("bvneg", [Bits(x, w)]) => Some(Bits(x.wrapping_neg() & mask(*w), *w)),
        ("concat", [Bits(x, w), Bits(y, v)]) if w + v <= 128 => Some(Bits((x << v) | y, w + v)),
        (_, [Bits(x, w), Bits(y, v)]) if w == v => {
            let (x, y, w) = (*x, *y, *w);
            let m = mask(w);
            let (sx, sy) = (signed(x, w), signed(y, w));
            Some(match op {
                "bvadd" => Bits(x.wrapping_add(y) & m, w),
                "bvsub" => Bits(x.wrapping_sub(y) & m, w),
                "bvmul" => Bits(x.wrapping_mul(y) & m, w),
                "bvand" => Bits(x & y, w),
                "bvor" => Bits(x | y, w),
                "bvxor" => Bits(x ^ y, w),
                "bvshl" => Bits(if y >= u128::from(w) { 0 } else { (x << y) & m }, w),
                "bvlshr" => Bits(if y >= u128::from(w) { 0 } else { x >> y }, w),
                "bvashr" => {
                    let shift = y.min(u128::from(w) - 1) as u32;
                    let shifted = if y >= u128::from(w) {
                        if sx < 0 { -1 } else { 0 }
                    } else {
                        sx >> shift
                    };
                    Bits((shifted as u128) & m, w)
                }
                // Division by zero has special SMT-LIB values: not folded.
                "bvudiv" if y != 0 => Bits(x / y, w),
                "bvurem" if y != 0 => Bits(x % y, w),
                "bvult" => Bool(x < y),
                "bvule" => Bool(x <= y),
                "bvugt" => Bool(x > y),
                "bvuge" => Bool(x >= y),
                "bvslt" => Bool(sx < sy),
                "bvsle" => Bool(sx <= sy),
                "bvsgt" => Bool(sx > sy),
                "bvsge" => Bool(sx >= sy),
                _ => return None,
            })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::bv;

    #[test]
    fn folds_ground_terms_and_leaves_symbols() {
        assert_eq!(
            fold(&format!("(bvadd {} {})", bv(255, 8), bv(2, 8))),
            Some(bv(1, 8))
        );
        assert_eq!(
            fold(&format!("(bvslt {} {})", bv(255, 8), bv(0, 8))),
            Some("true".into())
        );
        assert_eq!(
            fold(&format!("((_ sign_extend 8) {})", bv(0x80, 8))),
            Some(bv(0xff80, 16))
        );
        assert_eq!(
            fold(&format!(
                "(ite (= {} {}) {} {})",
                bv(1, 4),
                bv(1, 4),
                bv(1, 1),
                bv(0, 1)
            )),
            Some(bv(1, 1))
        );
        assert_eq!(fold(&format!("(bvadd v1 {})", bv(2, 8))), None);
        assert_eq!(fold(&format!("(bvudiv {} {})", bv(2, 8), bv(0, 8))), None);
        assert_eq!(fold(&format!("(select a {})", bv(0, 64))), None);
        assert_eq!(fold("(and true (not false))"), Some("true".into()));
        // Out-of-range literal values wrap as SMT-LIB defines.
        assert_eq!(fold("(bvult (_ bv300 8) (_ bv50 8))"), Some("true".into()));
    }

    /// Every operator against Z3 on pseudo-random operands and widths.
    #[test]
    fn agrees_with_z3() {
        let mut solver = crate::solver::Solver::new(5000).expect("Z3 required");
        let mut seed = 0x9e37_79b9_7f4a_7c15u64;
        let mut next = || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        let ops = [
            "bvadd", "bvsub", "bvmul", "bvand", "bvor", "bvxor", "bvshl", "bvlshr", "bvashr",
            "bvudiv", "bvurem", "bvult", "bvule", "bvugt", "bvuge", "bvslt", "bvsle", "bvsgt",
            "bvsge", "=",
        ];
        for round in 0..400 {
            let width = [1u32, 7, 8, 16, 33, 64, 128][round % 7];
            let pick = |r: u64| -> u128 {
                match r % 5 {
                    0 => 0,
                    1 => mask(width),
                    2 => 1u128 << (r as u32 % width),
                    _ => (u128::from(r) | (u128::from(r) << 64)) & mask(width),
                }
            };
            let (a, b) = (pick(next()), pick(next() >> 3));
            let op = ops[next() as usize % ops.len()];
            let mut term = format!("({op} {} {})", bv(a, width), bv(b, width));
            if round % 5 == 1 && width > 1 {
                let high = (next() as u32) % width;
                term = format!("((_ extract {high} 0) {term})");
                if op.starts_with("bvu") || op.starts_with("bvs") || op == "=" {
                    continue;
                }
            }
            if round % 3 == 0 {
                let k = 1 + (next() % 8) as u32;
                if width + k <= 128 && !op.starts_with("bvu") && !op.starts_with("bvs") && op != "="
                {
                    term = format!("((_ sign_extend {k}) {term})");
                }
            }
            let Some(folded) = fold(&term) else {
                assert!(
                    op.ends_with("div") || op.ends_with("rem"),
                    "{term} did not fold"
                );
                continue;
            };
            // Through a declared symbol, so Z3 (not the folder) evaluates it.
            let name = format!("g{round}");
            let sort = if folded.starts_with("(_ bv") {
                let width = folded.rsplit(' ').next().unwrap().trim_end_matches(')');
                format!("(_ BitVec {width})")
            } else {
                "Bool".into()
            };
            solver.declare(&name, &sort).unwrap();
            let before = solver.checks;
            assert_eq!(
                solver
                    .check(
                        &[format!("(= {name} {term})")],
                        &format!("(not (= {name} {folded}))")
                    )
                    .unwrap(),
                crate::solver::Sat::No,
                "{term} folded to {folded}"
            );
            assert!(solver.checks > before, "Z3 was not consulted");
        }
    }
}
