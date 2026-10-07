//! Per-byte definedness of integers read from memory. LLVM leaves unwritten
//! bytes undefined bit by bit, so a wide load that copies a partly
//! initialized struct (padding, an enum's unused payload, `MaybeUninit`) is
//! not poison as a whole: the defined bytes stay usable after a store,
//! `trunc`, constant shift or constant mask. A poison byte, however, poisons
//! the whole loaded integer (LangRef), so a `Kind::Bytes` value carries one
//! undef-freedom condition per byte (little-endian) plus one poison condition
//! for the whole value, and otherwise behaves as an integer whose `defined`
//! is their conjunction. Every other operation keeps that coarse, stricter
//! meaning.

use crate::value::{Kind, Value, and, bv, not};

/// An integer of `flags.len()` bytes with per-byte definedness.
pub fn bytes(expr: String, flags: Vec<String>, poison: String) -> Value {
    let mut all = vec![not(&poison)];
    all.extend(flags.iter().cloned());
    Value {
        defined: and(&all),
        expr,
        kind: Kind::Bytes { flags, poison },
    }
}

/// `value` made poison where `condition` fails. For per-byte values the
/// condition joins the whole-value poison, so stores and copies keep it.
pub fn restrict(value: Value, condition: &str) -> Value {
    if condition == "true" {
        return value;
    }
    match value.kind {
        Kind::Bytes { flags, poison } => bytes(
            value.expr,
            flags,
            crate::value::or(&[poison, not(condition)]),
        ),
        kind => Value {
            defined: and(&[value.defined, condition.to_owned()]),
            kind,
            expr: value.expr,
        },
    }
}

/// `select` with undef among its arms. A selected undef stays undef (any
/// value at each later use); a poison condition poisons the result either
/// way. An undef condition over two equal arms (of any kind, undef choices
/// too) has no choice to make; otherwise it counts as poison (strict).
pub fn select(
    cond: &str,
    defined: &str,
    undef_condition: bool,
    yes: Value,
    no: Value,
) -> Result<Value, String> {
    let undef = |v: &Value| matches!(v.kind, Kind::Undef(_));
    let choice = |v: &Value| matches!(v.kind, Kind::UndefChoice { .. });
    if undef_condition && yes == no {
        return Ok(yes);
    }
    if choice(&yes) || choice(&no) {
        return select_choices(cond, defined, yes, no);
    }
    let choose = |condition: String, other: Value| Value {
        expr: String::new(),
        kind: Kind::UndefChoice {
            condition,
            other: Box::new(other),
        },
        defined: "true".into(),
    };
    Ok(match (undef(&yes), undef(&no)) {
        (false, false) => crate::value::choose(cond, defined, yes, no)?,
        (true, true) if defined == "true" => yes,
        // Undef where the condition is defined, else poison. Only whole
        // bytes: an observed undef of another width counts as poison, which
        // would turn the undef case into a false counterexample.
        (true, true) => {
            let Kind::Undef(width) = yes.kind else {
                return Err("select of undef arms without a width".into());
            };
            if !width.is_multiple_of(8) {
                return Err("select of undef on a poison condition".into());
            }
            choose(
                defined.to_owned(),
                Value::bits(bv(0, width), width, "false".into()),
            )
        }
        (yes_undef, _) => {
            let (when, other) = if yes_undef {
                (cond.to_owned(), no)
            } else {
                (not(cond), yes)
            };
            choose(and(&[defined.to_owned(), when]), restrict(other, defined))
        }
    })
}

/// `select` where an arm is itself a partial undef choice (a select of
/// selects over undef): undef where the chosen arm is undef, otherwise the
/// chosen arm's value; a poison condition poisons the result.
pub fn select_choices(
    condition: &str,
    defined: &str,
    yes: Value,
    no: Value,
) -> Result<Value, String> {
    let split = |v: Value| match v.kind {
        Kind::UndefChoice { condition, other } => (condition, Some(*other)),
        Kind::Undef(_) => ("true".to_owned(), None),
        _ => ("false".to_owned(), Some(v)),
    };
    let (undef_yes, yes) = split(yes);
    let (undef_no, no) = split(no);
    // A wholly undef arm needs no value: the other arm's stands in.
    let (yes, no) = match (yes, no) {
        (Some(y), Some(n)) => (y, n),
        (Some(y), None) => (y.clone(), y),
        (None, Some(n)) => (n.clone(), n),
        (None, None) => return Err("select of two undef choices".into()),
    };
    let when = if undef_yes == undef_no {
        undef_yes
    } else {
        format!("(ite {condition} {undef_yes} {undef_no})")
    };
    let other = crate::value::choose(condition, defined, yes, no)?;
    Ok(Value {
        expr: String::new(),
        kind: Kind::UndefChoice {
            condition: and(&[defined.to_owned(), when]),
            other: Box::new(restrict(other, defined)),
        },
        defined: "true".into(),
    })
}

/// Per-byte flags and whole-value poison of an integer (coarse for a plain
/// value: all bytes defined, poison when undefined).
fn parts(value: &Value) -> Result<(Vec<String>, String), String> {
    match &value.kind {
        Kind::Bytes { flags, poison } => Ok((flags.clone(), poison.clone())),
        _ => {
            let width = value.width()?;
            if !width.is_multiple_of(8) {
                return Err("byte flags of a non-byte integer".into());
            }
            Ok((
                vec!["true".into(); (width / 8) as usize],
                not(&value.defined),
            ))
        }
    }
}

/// `select` between integers where one side has per-byte definedness; a
/// poison condition poisons the whole result.
pub fn choose(
    condition: &str,
    defined: &str,
    expr: String,
    yes: &Value,
    no: &Value,
) -> Result<Value, String> {
    let (yes_flags, yes_poison) = parts(yes)?;
    let (no_flags, no_poison) = parts(no)?;
    let flags = yes_flags
        .iter()
        .zip(&no_flags)
        .map(|(y, n)| {
            if y == n {
                y.clone()
            } else {
                format!("(ite {condition} {y} {n})")
            }
        })
        .collect();
    let poison = crate::value::or(&[
        not(defined),
        format!("(ite {condition} {yes_poison} {no_poison})"),
    ]);
    Ok(bytes(expr, flags, poison))
}

/// Byte pieces `(expr, 8, defined)` of a per-byte value, least significant
/// first, for reinterpretation as lanes.
pub fn pieces(value: &Value) -> Option<Vec<(String, u32, String)>> {
    let Kind::Bytes { flags, poison } = &value.kind else {
        return None;
    };
    Some(
        flags
            .iter()
            .enumerate()
            .map(|(i, flag)| {
                let low = 8 * i;
                (
                    format!("((_ extract {} {low}) {})", low + 7, value.expr),
                    8,
                    and(&[not(poison), flag.clone()]),
                )
            })
            .collect(),
    )
}

/// `count` undef bytes: what `store undef` leaves in memory.
pub fn undef(count: usize) -> Value {
    bytes(
        bv(0, 8 * count as u32),
        vec!["false".into(); count],
        "false".into(),
    )
}

/// `result` of a cast, with exact byte flags when `source` has them.
pub fn cast(opcode: &str, source: &Value, new: u32, result: Value) -> Result<Value, String> {
    let Kind::Bytes { flags, poison } = &source.kind else {
        return Ok(result);
    };
    if result.defined != source.defined {
        // An `nneg` condition applies to the whole value.
        return Ok(result);
    }
    let whole = (new / 8) as usize;
    match opcode {
        // The low bytes survive; a partial byte needs the byte it lies in.
        "trunc" if new.is_multiple_of(8) && whole > 1 => {
            Ok(bytes(result.expr, flags[..whole].to_vec(), poison.clone()))
        }
        "trunc" => {
            let mut kept = vec![not(poison)];
            kept.extend_from_slice(&flags[..new.div_ceil(8) as usize]);
            Ok(Value::bits(result.expr, new, and(&kept)))
        }
        "zext" | "sext" if new.is_multiple_of(8) => {
            // Zero bits are defined; copies of the sign bit share its byte.
            let fill = if opcode == "zext" {
                "true".to_owned()
            } else {
                flags.last().cloned().unwrap_or_else(|| "true".into())
            };
            let mut out = flags.clone();
            out.resize(whole, fill);
            Ok(bytes(result.expr, out, poison.clone()))
        }
        _ => Ok(result),
    }
}

/// `result` of `left OP right`, with exact byte flags when an operand carries
/// them: byte-aligned constant shifts (also `nuw`/`nsw`/`exact`, whose checks
/// need the bytes they inspect to be defined) and `and`/`or`/`xor` with any
/// second operand (also `or disjoint`). rustc's SROA rebuilds an enum from a
/// tag byte and a partly uninitialized payload exactly this way
/// (`zext`, `shl nuw 8`, `or disjoint`, `trunc`). Anything else keeps the
/// coarse result.
pub fn binary(
    opcode: &str,
    flags: &[&str],
    left: &Value,
    right: &Value,
    result: Value,
) -> Result<Value, String> {
    let bytewise = |v: &Value| matches!(v.kind, Kind::Bytes { .. });
    if matches!(opcode, "and" | "or" | "xor") && (bytewise(left) || bytewise(right)) {
        return bitwise(opcode, flags, left, right, result);
    }
    let Kind::Bytes {
        flags: input,
        poison,
    } = &left.kind
    else {
        return Ok(result);
    };
    if right.defined != "true" {
        return Ok(result);
    }
    let Some(constant) = crate::heap::literal(&right.expr) else {
        return Ok(result);
    };
    let count = input.len();
    if !matches!(opcode, "lshr" | "shl" | "ashr")
        || constant % 8 != 0
        || constant as usize / 8 >= count
    {
        return Ok(result);
    }
    let shift = constant as usize / 8;
    let fill = if opcode == "ashr" {
        input[count - 1].clone()
    } else {
        "true".to_owned()
    };
    let mut out = vec![fill; count];
    if opcode == "shl" {
        out[shift..].clone_from_slice(&input[..count - shift]);
    } else {
        out[..count - shift].clone_from_slice(&input[shift..]);
    }
    if flags.is_empty() {
        return Ok(bytes(result.expr, out, poison.clone()));
    }
    // The flag checks read the shifted-out bytes (and for nsw the byte that
    // becomes the sign): they must be defined for the check to mean anything.
    let mut inspected: Vec<String> = Vec::new();
    for flag in flags {
        let range = match (*flag, opcode) {
            ("nuw", "shl") => count - shift..count,
            ("nsw", "shl") => count - shift - 1..count,
            ("exact", "lshr" | "ashr") => 0..shift,
            _ => return Ok(result),
        };
        inspected.extend(input[range].iter().cloned());
    }
    let strict = crate::value::binary(opcode, flags, &whole(left), &whole(right))?.defined;
    inspected.push(strict);
    let poison = crate::value::or(&[poison.clone(), not(&and(&inspected))]);
    Ok(bytes(result.expr, out, poison))
}

/// The same value with every bit treated as defined, to read off the
/// conditions an operation's flags add on top of operand definedness.
fn whole(value: &Value) -> Value {
    Value::bits(
        value.expr.clone(),
        value.width().unwrap_or(0),
        "true".into(),
    )
}

/// Per-byte flags and whole-value poison of an operand; a value without byte
/// flags is defined or poison as a whole.
fn per_byte(value: &Value, count: usize) -> (Vec<String>, String) {
    match &value.kind {
        Kind::Bytes { flags, poison } if flags.len() == count => (flags.clone(), poison.clone()),
        _ => (vec!["true".into(); count], not(&value.defined)),
    }
}

fn byte(expr: &str, i: usize) -> String {
    format!("((_ extract {} {}) {expr})", 8 * i + 7, 8 * i)
}

/// `and`/`or`/`xor` byte by byte. A result byte is defined when both input
/// bytes are, or when one is a defined absorbing byte (0 for `and`, 0xff for
/// `or`). `or disjoint` additionally needs, per byte, either both bytes
/// defined with no common bit or one side a defined zero.
fn bitwise(
    opcode: &str,
    flags: &[&str],
    left: &Value,
    right: &Value,
    result: Value,
) -> Result<Value, String> {
    let width = left.width()?;
    if width != right.width()? || !width.is_multiple_of(8) {
        return Ok(result);
    }
    if flags.iter().any(|f| !(*f == "disjoint" && opcode == "or")) {
        return Ok(result);
    }
    // A constant mask keeps the existing exact flags.
    if flags.is_empty()
        && opcode != "xor"
        && right.defined == "true"
        && crate::heap::literal(&right.expr).is_some()
        && matches!(left.kind, Kind::Bytes { .. })
    {
        return constant_mask(opcode, left, right, result);
    }
    let count = (width / 8) as usize;
    let (lf, lp) = per_byte(left, count);
    let (rf, rp) = per_byte(right, count);
    let mut out = Vec::with_capacity(count);
    let mut disjoint = Vec::new();
    for i in 0..count {
        let (l, r) = (byte(&left.expr, i), byte(&right.expr, i));
        let both = and(&[lf[i].clone(), rf[i].clone()]);
        let zero = bv(0, 8);
        let absorbing = match opcode {
            "and" => Some(zero.clone()),
            "or" => Some(bv(0xff, 8)),
            _ => None,
        };
        out.push(match absorbing {
            Some(a) => crate::value::or(&[
                both.clone(),
                and(&[lf[i].clone(), format!("(= {l} {a})")]),
                and(&[rf[i].clone(), format!("(= {r} {a})")]),
            ]),
            None => both.clone(),
        });
        if !flags.is_empty() {
            disjoint.push(crate::value::or(&[
                and(&[both, format!("(= (bvand {l} {r}) {zero})")]),
                and(&[lf[i].clone(), format!("(= {l} {zero})")]),
                and(&[rf[i].clone(), format!("(= {r} {zero})")]),
            ]));
        }
    }
    let poison = crate::value::or(&[lp, rp, not(&and(&disjoint))]);
    Ok(bytes(result.expr, out, poison))
}

/// `and`/`or` with a constant: a constant 0 byte in `and` (0xff in `or`) fixes
/// that result byte; the others keep the input byte's flag.
fn constant_mask(
    opcode: &str,
    left: &Value,
    right: &Value,
    result: Value,
) -> Result<Value, String> {
    let Kind::Bytes {
        flags: input,
        poison,
    } = &left.kind
    else {
        return Ok(result);
    };
    let Some(constant) = crate::heap::literal(&right.expr) else {
        return Ok(result);
    };
    let fixed = if opcode == "and" { 0 } else { 0xff };
    let out = (0..input.len())
        .map(|i| {
            if (constant >> (8 * i)) & 0xff == fixed {
                "true".to_owned()
            } else {
                input[i].clone()
            }
        })
        .collect();
    Ok(bytes(result.expr, out, poison.clone()))
}

/// Per-byte initialization and the poison condition for storing `value` of
/// `count` bytes: a copy keeps its bytes; any other undefined value counts
/// as poison in every byte (stricter than undef).
pub fn store_flags(value: &Value, count: usize) -> Result<(Vec<String>, String), String> {
    let (flags, poison) = match &value.kind {
        Kind::Bytes { flags, poison } => (flags.clone(), poison.clone()),
        _ => (vec![value.defined.clone(); count], not(&value.defined)),
    };
    if flags.len() != count {
        return Err("stored value width differs".into());
    }
    Ok((flags, poison))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn loaded(poison: &str) -> Value {
        bytes(
            "x".into(),
            vec!["a".into(), "b".into(), "c".into(), "d".into()],
            poison.into(),
        )
    }
    fn flags(value: &Value) -> Vec<String> {
        match &value.kind {
            Kind::Bytes { flags, .. } => flags.clone(),
            _ => panic!("expected bytes"),
        }
    }
    fn coarse(width: u32) -> Value {
        Value::bits("r".into(), width, "coarse".into())
    }

    #[test]
    fn bytes_follow_casts_shifts_and_masks() {
        let x = loaded("false");
        let low = cast(
            "trunc",
            &x,
            8,
            Value::bits("t".into(), 8, x.defined.clone()),
        )
        .unwrap();
        assert_eq!(low.defined, "a");
        let half = cast(
            "trunc",
            &x,
            16,
            Value::bits("t".into(), 16, x.defined.clone()),
        )
        .unwrap();
        assert_eq!(flags(&half), ["a", "b"]);
        let bit = cast(
            "trunc",
            &x,
            1,
            Value::bits("t".into(), 1, x.defined.clone()),
        )
        .unwrap();
        assert_eq!(bit.defined, "a");
        let sixteen = Value::bits(bv(16, 32), 32, "true".into());
        let shifted = binary("lshr", &[], &x, &sixteen, coarse(32)).unwrap();
        assert_eq!(flags(&shifted), ["c", "d", "true", "true"]);
        let left = binary("shl", &[], &x, &sixteen, coarse(32)).unwrap();
        assert_eq!(flags(&left), ["true", "true", "a", "b"]);
        let mask = Value::bits(bv(0xff, 32), 32, "true".into());
        let masked = binary("and", &[], &x, &mask, coarse(32)).unwrap();
        assert_eq!(flags(&masked), ["a", "true", "true", "true"]);
        // `exact` keeps byte flags; the dropped low bytes must be defined.
        let exact = binary("lshr", &["exact"], &x, &sixteen, coarse(32)).unwrap();
        assert_eq!(flags(&exact), ["c", "d", "true", "true"]);
        assert!(exact.defined.contains('a') && exact.defined.contains('b'));
        // A non-byte-aligned shift keeps the coarse result.
        let odd = Value::bits(bv(3, 32), 32, "true".into());
        assert_eq!(
            binary("shl", &[], &x, &odd, coarse(32)).unwrap().defined,
            "coarse"
        );
        let widened = cast(
            "sext",
            &x,
            48,
            Value::bits("w".into(), 48, x.defined.clone()),
        )
        .unwrap();
        assert_eq!(flags(&widened), ["a", "b", "c", "d", "d", "d"]);
    }
    #[test]
    fn bitwise_bytes_combine_and_absorb() {
        let x = loaded("false");
        let y = bytes(
            "y".into(),
            vec!["e".into(), "f".into(), "g".into(), "h".into()],
            "q".into(),
        );
        let x_or_y = binary("or", &[], &x, &y, coarse(32)).unwrap();
        // Defined where both are, or one side is a defined 0xff byte.
        assert!(flags(&x_or_y)[0].contains('a') && flags(&x_or_y)[0].contains('e'));
        let plain = Value::bits("v".into(), 32, "d".into());
        let x_xor_v = binary("xor", &[], &x, &plain, coarse(32)).unwrap();
        assert_eq!(flags(&x_xor_v), ["a", "b", "c", "d"]);
        // A plain operand's undefinedness is poison for the whole result.
        assert!(x_xor_v.defined.contains("(not d)"));
        // `disjoint` adds a per-byte overlap condition to the poison.
        let disjoint = binary("or", &["disjoint"], &x, &y, coarse(32)).unwrap();
        assert!(disjoint.defined.contains("bvand"));
        // Any other flag keeps the coarse result.
        assert_eq!(
            binary("and", &["nuw"], &x, &y, coarse(32)).unwrap().defined,
            "coarse"
        );
    }
    #[test]
    fn a_poison_byte_poisons_every_extraction() {
        let x = loaded("p");
        let low = cast(
            "trunc",
            &x,
            8,
            Value::bits("t".into(), 8, x.defined.clone()),
        )
        .unwrap();
        assert_eq!(low.defined, "(and (not p) a)");
        let mask = Value::bits(bv(0, 32), 32, "true".into());
        let masked = binary("and", &[], &x, &mask, coarse(32)).unwrap();
        assert_eq!(masked.defined, "(not p)");
        // Stored copies keep both; plain undefined values store as poison.
        assert_eq!(store_flags(&x, 4).unwrap().1, "p");
        let plain = Value::bits("v".into(), 16, "d".into());
        assert_eq!(
            store_flags(&plain, 2).unwrap(),
            (vec!["d".into(), "d".into()], "(not d)".into())
        );
        let hole = undef(2);
        assert_eq!(store_flags(&hole, 2).unwrap().1, "false");
    }
}
