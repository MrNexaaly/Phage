//! Atomic memory operations under the property's single thread. Orderings
//! and synchronization scopes constrain only what other threads observe, so
//! with one thread `load/store atomic` are ordinary accesses, `atomicrmw` is
//! a read-modify-write, `cmpxchg` a conditional store (a `weak` one may also
//! fail spuriously) and `fence` has no effect. Volatile and pointer-valued
//! read-modify-writes stay unsupported.

use crate::{
    engine::{Engine, State},
    ir::split,
    memory,
    value::{Kind, Value, and, bv},
};

const ORDERINGS: [&str; 6] = [
    "unordered",
    "monotonic",
    "acquire",
    "release",
    "acq_rel",
    "seq_cst",
];

/// `code` without its `syncscope(...)` and ordering words; the comma that
/// followed an ordering stays with the preceding operand.
pub fn without_orderings(code: &str) -> String {
    let mut kept: Vec<String> = Vec::new();
    for word in code.split_whitespace() {
        let bare = word.strip_suffix(',').unwrap_or(word);
        if ORDERINGS.contains(&bare) || word.starts_with("syncscope(") {
            if word.ends_with(',')
                && let Some(last) = kept.last_mut()
            {
                last.push(',');
            }
            continue;
        }
        kept.push(word.to_owned());
    }
    kept.join(" ")
}

/// `load atomic`/`store atomic` rewritten as the plain access.
pub fn plain_access(code: &str) -> Result<String, String> {
    let (opcode, rest) = code.split_once(' ').ok_or("atomic access malformed")?;
    let rest = rest.strip_prefix("atomic ").ok_or("not an atomic access")?;
    if rest.starts_with("volatile ") {
        return Err("volatile atomic accesses are unsupported".into());
    }
    Ok(without_orderings(&format!("{opcode} {rest}")))
}

/// Stated alignment; LLVM defaults an omitted one to the value's size.
fn align(args: &[&str], width: u32) -> u64 {
    args.iter()
        .find_map(|arg| arg.strip_prefix("align "))
        .and_then(|s| s.parse().ok())
        .unwrap_or(u64::from(width.div_ceil(8)))
}

impl Engine {
    /// The pointer and integer operand of a read-modify-write, the access
    /// checked and the old (initialized) contents loaded.
    fn atomic_old(
        &mut self,
        state: &mut State,
        args: &[&str],
        operands: usize,
    ) -> Result<(Value, Vec<Value>, Value), String> {
        if args.len() < operands + 1 {
            return Err("atomic operands missing".into());
        }
        let pointer = self.typed_operand(state, args[0])?;
        let pointer = self.dereference(state, &pointer)?;
        let mut values = Vec::new();
        for arg in &args[1..=operands] {
            let value = self.typed_operand(state, arg)?;
            if !matches!(
                value.kind,
                Kind::Bits(_) | Kind::PointerBits(_) | Kind::Bytes { .. }
            ) {
                return Err("only integer atomic read-modify-writes are supported".into());
            }
            values.push(value);
        }
        let width = values[0].width()?;
        let valid = memory::access_store(
            &state.memory,
            &pointer,
            u64::from(width.div_ceil(8)),
            align(args, width),
        )?;
        self.safety(state, &valid, "invalid atomic memory access")?;
        let old = memory::load(&state.memory, &pointer, width)?;
        self.safety(state, &old.defined, "uninitialized or poison memory read")?;
        Ok((pointer, values, old))
    }

    /// `atomicrmw OP ptr P, iN V, align A` (orderings already removed):
    /// stores `old OP V` and yields `old`.
    pub fn atomicrmw(&mut self, state: &mut State, rest: &str) -> Result<Value, String> {
        let (op, rest) = rest.split_once(' ').ok_or("atomicrmw operation missing")?;
        if op == "volatile" {
            return Err("volatile atomics are unsupported".into());
        }
        let args = split(rest);
        let (pointer, values, old) = self.atomic_old(state, &args, 1)?;
        let (a, b) = (&old.expr, &values[0].expr);
        let expr = match op {
            "xchg" => b.clone(),
            "add" => format!("(bvadd {a} {b})"),
            "sub" => format!("(bvsub {a} {b})"),
            "and" => format!("(bvand {a} {b})"),
            "nand" => format!("(bvnot (bvand {a} {b}))"),
            "or" => format!("(bvor {a} {b})"),
            "xor" => format!("(bvxor {a} {b})"),
            "max" => format!("(ite (bvsge {a} {b}) {a} {b})"),
            "min" => format!("(ite (bvsle {a} {b}) {a} {b})"),
            "umax" => format!("(ite (bvuge {a} {b}) {a} {b})"),
            "umin" => format!("(ite (bvule {a} {b}) {a} {b})"),
            _ => return Err(format!("unsupported atomicrmw {op}")),
        };
        let width = old.width()?;
        let defined = and(&[old.defined.clone(), values[0].defined.clone()]);
        // An exchange stores the operand itself, per-byte definedness intact.
        let stored = if op == "xchg" {
            values[0].clone()
        } else {
            Value::bits(expr, width, defined)
        };
        memory::store(&mut state.memory, &pointer, &stored)?;
        Ok(old)
    }

    /// `cmpxchg [weak] ptr P, iN C, iN N, align A` (orderings removed):
    /// yields `{ old, success }` and stores N only on success.
    pub fn cmpxchg(&mut self, state: &mut State, rest: &str) -> Result<Value, String> {
        let (weak, rest) = match rest.strip_prefix("weak ") {
            Some(rest) => (true, rest),
            None => (false, rest),
        };
        if rest.starts_with("volatile ") {
            return Err("volatile atomics are unsupported".into());
        }
        let args = split(rest);
        let (pointer, values, old) = self.atomic_old(state, &args, 2)?;
        let (expected, new) = (&values[0], &values[1]);
        // The comparison decides a store, so a poison comparand is refused.
        self.safety(state, &expected.defined, "poison cmpxchg comparand")?;
        let equal = format!("(= {} {})", old.expr, expected.expr);
        let success = if weak {
            // A weak exchange may fail even when the values are equal.
            let spurious = self.symbol("Bool")?;
            format!("(and {equal} {spurious})")
        } else {
            equal
        };
        let width = old.width()?;
        let expr = format!("(ite {success} {} {})", new.expr, old.expr);
        // Per byte: the new operand's flags on success, the old (checked
        // defined above) contents otherwise.
        let stored = if width.is_multiple_of(8) {
            let (flags, poison) = crate::partial::store_flags(new, (width / 8) as usize)?;
            let failed = crate::value::not(&success);
            let flags = flags
                .iter()
                .map(|f| crate::value::or(&[failed.clone(), f.clone()]))
                .collect();
            crate::partial::bytes(expr, flags, and(&[success.clone(), poison]))
        } else {
            Value::bits(
                expr,
                width,
                format!("(ite {success} {} {})", new.defined, old.defined),
            )
        };
        memory::store(&mut state.memory, &pointer, &stored)?;
        let flag = Value::bits(
            format!("(ite {success} {} {})", bv(1, 1), bv(0, 1)),
            1,
            "true".into(),
        );
        Ok(Value {
            expr: String::new(),
            kind: Kind::Aggregate(vec![old, flag]),
            defined: "true".into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orderings_and_scopes_are_dropped() {
        assert_eq!(
            plain_access("load atomic i8, ptr @x monotonic, align 1").unwrap(),
            "load i8, ptr @x, align 1"
        );
        assert_eq!(
            plain_access(
                "store atomic i64 %v, ptr %p syncscope(\"singlethread\") release, align 8"
            )
            .unwrap(),
            "store i64 %v, ptr %p, align 8"
        );
        assert_eq!(
            without_orderings("cmpxchg ptr %p, i32 0, i32 1 acq_rel monotonic, align 4"),
            "cmpxchg ptr %p, i32 0, i32 1, align 4"
        );
        assert!(plain_access("load atomic volatile i8, ptr @x seq_cst, align 1").is_err());
    }
}
