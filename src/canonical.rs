//! Canonical arithmetic terms. rustc inlines the same computation (SipHash
//! of one key in `insert`, `get` and `remove`, say) several times, and LLVM
//! reassociates each copy differently: `(k1 ^ x) ^ C1` in one place,
//! `(k1 ^ (x | B)) ^ C2` with `C1 = C2 ^ B` in another. Equal values with
//! different text defeat hash-consed definitions and, once the terms are
//! opaque (opaque.rs), even congruence. Two rewrites make such copies one
//! term:
//!
//! * commutative operators order their operands (literals last, so the
//!   per-byte mask logic in partial.rs still sees its constant on the right);
//! * XOR chains, including `or disjoint` (equal to XOR wherever defined; the
//!   disjointness stays in the definedness), become a sorted set of leaves
//!   plus one folded constant, rebuilt left to right. `x ^ x` cancels.
//!
//! Both rewrites are exact identities on defined values, so verdicts do not
//! change; opaque applications built here still record exact definitions.

use crate::{
    engine::{Engine, State},
    value::{Kind, Value, bv},
};

const COMMUTATIVE: [&str; 5] = ["add", "mul", "and", "or", "xor"];

/// A XOR of distinct non-literal leaves (sorted) and one constant.
#[derive(Clone, Debug, PartialEq)]
pub struct XorForm {
    leaves: Vec<String>,
    constant: u128,
}

impl XorForm {
    fn join(&self, other: &XorForm) -> XorForm {
        // Symmetric difference of two sorted sets: equal leaves cancel.
        let (mut a, mut b) = (
            self.leaves.iter().peekable(),
            other.leaves.iter().peekable(),
        );
        let mut leaves = Vec::new();
        loop {
            match (a.peek(), b.peek()) {
                (Some(x), Some(y)) if x == y => {
                    a.next();
                    b.next();
                }
                (Some(x), Some(y)) if x < y => leaves.push(a.next().unwrap().clone()),
                (Some(_), Some(_)) | (None, Some(_)) => leaves.push(b.next().unwrap().clone()),
                (Some(_), None) => leaves.push(a.next().unwrap().clone()),
                (None, None) => break,
            }
        }
        XorForm {
            leaves,
            constant: self.constant ^ other.constant,
        }
    }
}

/// Operands of a commutative operator in canonical order.
pub fn commute(opcode: &str, left: Value, right: Value) -> (Value, Value) {
    // A per-byte value counts as non-literal even when its bits are a
    // literal: partial.rs needs it on the left of a constant mask.
    let key = |v: &Value| {
        let bytes = matches!(v.kind, Kind::Bytes { .. });
        (
            !bytes && crate::heap::literal(&v.expr).is_some(),
            v.expr.clone(),
        )
    };
    if COMMUTATIVE.contains(&opcode) && key(&right) < key(&left) {
        (right, left)
    } else {
        (left, right)
    }
}

impl Engine {
    fn xor_form(&self, value: &Value) -> XorForm {
        if let Some(constant) = crate::heap::literal(&value.expr) {
            return XorForm {
                leaves: Vec::new(),
                constant,
            };
        }
        self.xor_forms.get(&value.expr).cloned().unwrap_or(XorForm {
            leaves: vec![value.expr.clone()],
            constant: 0,
        })
    }

    /// `exact` (the `xor`, or `or disjoint`, of `left` and `right`) as its
    /// canonical XOR term, or `None` when the rewrite does not apply.
    pub fn canonical_xor(
        &mut self,
        state: &mut State,
        opcode: &str,
        flags: &[&str],
        left: &Value,
        right: &Value,
        exact: &Value,
    ) -> Result<Option<Value>, String> {
        let applies = match opcode {
            "xor" => flags.is_empty(),
            "or" => flags == ["disjoint"],
            _ => false,
        };
        // Operands may carry per-byte definedness (loads); only the result's
        // definedness is kept, and XOR/or results are coarse `Bits`.
        if !applies || !matches!(exact.kind, Kind::Bits(_)) {
            return Ok(None);
        }
        let width = exact.width()?;
        let form = self.xor_form(left).join(&self.xor_form(right));
        let mut term: Option<String> = None;
        let constant = (form.constant != 0).then(|| bv(form.constant, width));
        for leaf in form.leaves.iter().cloned().chain(constant) {
            term = Some(match term {
                None => leaf,
                Some(acc) => self.xor_term(state, acc, leaf, width)?,
            });
        }
        let expr = term.unwrap_or_else(|| bv(0, width));
        self.xor_forms.insert(expr.clone(), form);
        Ok(Some(Value {
            expr,
            kind: Kind::Bits(width),
            defined: exact.defined.clone(),
        }))
    }

    /// One XOR application, opaque when it mixes random-derived data.
    fn xor_term(
        &mut self,
        state: &mut State,
        acc: String,
        leaf: String,
        width: u32,
    ) -> Result<String, String> {
        let exact = format!("(bvxor {acc} {leaf})");
        if !self.is_tainted(&acc) && !self.is_tainted(&leaf) {
            return Ok(exact);
        }
        let a = Value::bits(acc, width, "true".into());
        let b = Value::bits(leaf, width, "true".into());
        let exact = Value::bits(exact, width, "true".into());
        Ok(self.opaque(state, "xor", &[&a, &b], exact)?.expr)
    }

    /// Carries a canonical form over to the solver name defined as `expr`.
    pub fn alias_form(&mut self, expr: &str, name: &str) {
        if let Some(form) = self.xor_forms.get(expr).cloned() {
            self.xor_forms.insert(name.to_owned(), form);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn form(leaves: &[&str], constant: u128) -> XorForm {
        XorForm {
            leaves: leaves.iter().map(|s| s.to_string()).collect(),
            constant,
        }
    }
    #[test]
    fn xor_forms_cancel_and_fold() {
        let joined = form(&["a", "k"], 0x75).join(&form(&["b", "k"], 0x74));
        assert_eq!(joined, form(&["a", "b"], 1));
        assert_eq!(form(&["x"], 3).join(&form(&["x"], 3)), form(&[], 0));
    }
    #[test]
    fn commuting_keeps_literals_right() {
        let x = Value::bits("x".into(), 8, "true".into());
        let c = Value::bits(bv(3, 8), 8, "true".into());
        let (l, r) = commute("and", c.clone(), x.clone());
        assert_eq!((l.expr.as_str(), r.expr.as_str()), ("x", "(_ bv3 8)"));
        // Not commutative: order kept.
        let (l, _) = commute("sub", c.clone(), x);
        assert_eq!(l.expr, "(_ bv3 8)");
        // Per-byte values stay left of a constant even with literal bits.
        let partly = crate::partial::bytes(bv(7, 8), vec!["d".into()], "false".into());
        let (l, r) = commute(
            "and",
            partly.clone(),
            Value::bits(bv(0, 8), 8, "true".into()),
        );
        assert!(matches!(l.kind, Kind::Bytes { .. }) && r.expr == "(_ bv0 8)");
        let (l, _) = commute("and", c, partly);
        assert!(matches!(l.kind, Kind::Bytes { .. }));
    }
}
