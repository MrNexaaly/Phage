//! Opaque mixing of values derived from the OS random source. HashMap's
//! SipHash keys come from `getrandom`; exact 64-bit add/xor/rotate chains over
//! unknown keys are what make hash-table queries slow, yet a correct program
//! cannot depend on the particular hash function. Values computed from the
//! random bytes are *tainted*; an add/sub/mul/xor/or/shl or rotate with a
//! tainted operand becomes an uninterpreted function of the same operands
//! (one function per operator and width), so equal inputs still give equal
//! results by congruence while the mixing itself costs the solver nothing.
//! Masks, shifts and truncations of a tainted value stay exact and end the
//! taint, so bucket indices derived from a hash are reasoned about exactly.
//!
//! Soundness: the real operator is one interpretation of its function, so an
//! abstract unsat is a real unsat (proofs stand). Each abstraction also
//! records its exact definition on the path; a satisfiable violation is
//! confirmed with those definitions before it becomes a counterexample, and
//! an obligation that holds exactly lets the path continue. Taint precision
//! affects speed only: less taint means more exact reasoning, never a wrong
//! verdict.
//!
//! A whole opaque computation is one function of its inputs: each opaque
//! term keeps its *shape* (the operator tree over leaf positions, interned)
//! and its *leaves* (the non-opaque operands it reads, deduplicated), and is
//! the application `(ru_s<shape> leaves...)`. All of SipHash over the key
//! words and one input is a single application, so the solver relates the
//! few hashes a query mentions instead of hundreds of round operations
//! pairwise (Ackermann constraints grow quadratically in applications). Equal
//! shape and leaves mean the same computation on the same values, so the
//! real function remains one interpretation of every symbol.
//!
//! Confirming a candidate with the full exact definitions means inverting
//! SipHash, which bit-blasting rarely finishes. Most hash-table bugs hold for
//! every key, so confirmation first tries concrete witnesses: the abstract
//! model's inputs with the random bytes fixed (the model's own, then a few
//! deterministic samples). Everything is then ground and decided by
//! propagation; a satisfiable witness is a real counterexample under exact
//! semantics. Only when no witness works does the full exact query run.

use crate::{
    engine::{Engine, State},
    solver::Sat,
    value::{Kind, Value},
};

/// Operators replaced by an uninterpreted function on tainted operands.
const MIXING: [&str; 6] = ["add", "sub", "mul", "xor", "or", "shl"];
/// Sampled random-source values tried after the abstract model's own.
const SAMPLES: u64 = 8;

/// Interned shapes and the shape and leaves of every opaque term.
#[derive(Default)]
pub struct Shapes {
    ids: std::collections::HashMap<String, usize>,
    terms: std::collections::HashMap<String, Term>,
}

#[derive(Clone)]
struct Term {
    shape: usize,
    /// Leaf expressions and widths, by position.
    leaves: Vec<(String, u32)>,
}

impl Shapes {
    /// Carries a term's shape over to the solver name defined as `expr`.
    pub fn alias(&mut self, expr: &str, name: &str) {
        if let Some(term) = self.terms.get(expr).cloned() {
            self.terms.insert(name.to_owned(), term);
        }
    }
}

fn position(leaves: &mut Vec<(String, u32)>, leaf: (String, u32)) -> usize {
    match leaves.iter().position(|l| l.0 == leaf.0) {
        Some(index) => index,
        None => {
            leaves.push(leaf);
            leaves.len() - 1
        }
    }
}

impl Engine {
    /// True when `expr` mentions a symbol derived from the random source.
    pub fn is_tainted(&self, expr: &str) -> bool {
        !self.tainted.is_empty()
            && expr
                .split(|c: char| c == '(' || c == ')' || c.is_whitespace())
                .any(|token| self.tainted.contains(token))
    }

    /// Marks a materialized value as derived from the random source.
    pub fn taint(&mut self, value: &Value) {
        if crate::setting("TRACE_TAINT").is_some() {
            eprintln!("taint {} ({})", value.expr, self.solver.context);
        }
        if !value.expr.is_empty() && crate::heap::literal(&value.expr).is_none() {
            self.tainted.insert(value.expr.clone());
        }
    }

    /// An uninterpreted function application standing for `exact`, whose
    /// definition is recorded on the path for counterexample confirmation.
    pub(crate) fn opaque(
        &mut self,
        state: &mut State,
        name: &str,
        arguments: &[&Value],
        exact: Value,
    ) -> Result<Value, String> {
        let width = exact.width()?;
        // The operator over its arguments' shapes, leaves renumbered into
        // one deduplicated list; literals are part of the shape.
        let mut leaves: Vec<(String, u32)> = Vec::new();
        let mut parts = Vec::new();
        for argument in arguments {
            if let Some(term) = self.shapes.terms.get(&argument.expr).cloned() {
                let map: Vec<String> = term
                    .leaves
                    .into_iter()
                    .map(|leaf| position(&mut leaves, leaf).to_string())
                    .collect();
                parts.push(format!("s{}[{}]", term.shape, map.join(",")));
            } else if crate::heap::literal(&argument.expr).is_some() {
                parts.push(argument.expr.clone());
            } else {
                let at = position(&mut leaves, (argument.expr.clone(), argument.width()?));
                parts.push(format!("@{at}"));
            }
        }
        let key = format!("{name}:{width}({})", parts.join(" "));
        let next = self.shapes.ids.len();
        let shape = *self.shapes.ids.entry(key).or_insert(next);
        let sorts: Vec<String> = leaves
            .iter()
            .map(|(_, w)| format!("(_ BitVec {w})"))
            .collect();
        let function = format!("ru_s{shape}_{width}");
        if self.opaque_functions.insert(function.clone()) {
            self.solver
                .declare_function(&function, &sorts, &format!("(_ BitVec {width})"))?;
            self.assumptions.insert(
                "values mixed from the OS random source (e.g. hash keys) are uninterpreted \
                 functions of their inputs; counterexamples were confirmed with exact semantics"
                    .into(),
            );
        }
        let operands: Vec<&str> = leaves.iter().map(|(l, _)| l.as_str()).collect();
        let application = format!("({function} {})", operands.join(" "));
        state
            .abstractions
            .push(format!("(= {application} {})", exact.expr));
        self.shapes
            .terms
            .insert(application.clone(), Term { shape, leaves });
        // Only the value is abstracted: per-byte definedness (a partly
        // initialized load through a constant mask or shift) stays exact.
        Ok(match exact.kind {
            Kind::Bytes { flags, poison } => crate::partial::bytes(application, flags, poison),
            _ => Value {
                expr: application,
                kind: Kind::Bits(width),
                defined: exact.defined,
            },
        })
    }

    /// `left OP right` made opaque when it mixes a tainted value.
    pub fn opaque_binary(
        &mut self,
        state: &mut State,
        opcode: &str,
        flags: &[&str],
        left: &Value,
        right: &Value,
        exact: Value,
    ) -> Result<(Value, bool), String> {
        let mixing = MIXING.contains(&opcode)
            && flags.is_empty()
            && (self.is_tainted(&left.expr) || self.is_tainted(&right.expr));
        if !mixing {
            return Ok((exact, false));
        }
        Ok((self.opaque(state, opcode, &[left, right], exact)?, true))
    }

    /// A rotate (`llvm.fshl`/`llvm.fshr`) made opaque when it mixes a tainted
    /// value; `None` leaves the exact model.
    pub fn opaque_rotate(
        &mut self,
        state: &mut State,
        callee: &str,
        arguments: &[Value],
        exact: &Value,
    ) -> Result<Option<Value>, String> {
        let name = if callee.starts_with("llvm.fshl.") {
            "fshl"
        } else if callee.starts_with("llvm.fshr.") {
            "fshr"
        } else {
            return Ok(None);
        };
        if arguments.len() != 3 || !arguments[..2].iter().any(|a| self.is_tainted(&a.expr)) {
            return Ok(None);
        }
        let refs: Vec<&Value> = arguments.iter().collect();
        Ok(Some(self.opaque(state, name, &refs, exact.clone())?))
    }

    /// Whether `condition` can hold on this path with exact semantics: the
    /// abstract answer, confirmed with the recorded definitions when it is
    /// satisfiable and the path used opaque functions. Also returns the
    /// constraints that decided it, for building the counterexample model.
    pub fn exact_check(
        &mut self,
        state: &State,
        condition: &str,
    ) -> Result<(Sat, Vec<String>), String> {
        let abstract_answer = self.solver.check(&state.constraints, condition)?;
        if abstract_answer != Sat::Yes || state.abstractions.is_empty() {
            return Ok((abstract_answer, state.constraints.clone()));
        }
        if crate::setting("TRACE_TAINT").is_some() {
            let inputs = self.inputs.clone();
            let model = self.solver.model(&state.constraints, condition, &inputs)?;
            eprintln!(
                "abstract candidate ({}): {condition}\n{model}",
                self.solver.context
            );
        }
        let exact = self.exact_constraints(state);
        if let Some(witness) = self.witness(state, &exact, condition)? {
            return Ok((Sat::Yes, witness));
        }
        let answer = self.solver.check(&exact, condition)?;
        Ok((answer, exact))
    }

    /// Exact constraints with the inputs and random bytes fixed to values
    /// under which `condition` holds, if one of the tried values does.
    fn witness(
        &mut self,
        state: &State,
        exact: &[String],
        condition: &str,
    ) -> Result<Option<Vec<String>>, String> {
        if self.random_sources.is_empty() {
            return Ok(None);
        }
        let randoms: Vec<String> = self.random_sources.iter().map(|(n, _)| n.clone()).collect();
        let mut names = self.inputs.clone();
        names.extend(randoms.iter().cloned());
        let model = self.solver.model(&state.constraints, condition, &names)?;
        let values = model_values(&model, &names);
        let mut fixed = exact.to_vec();
        fixed.extend(
            values
                .iter()
                .filter(|(name, _)| self.inputs.contains(name))
                .map(|(name, value)| format!("(= {name} {value})")),
        );
        for sample in 0..=SAMPLES {
            let mut candidate = fixed.clone();
            for (index, (name, width)) in self.random_sources.iter().enumerate() {
                let value = match sample {
                    0 => match values.iter().find(|(n, _)| n == name) {
                        Some((_, value)) => value.clone(),
                        None => continue,
                    },
                    _ => sampled(*width, sample, index as u64),
                };
                candidate.push(format!("(= {name} {value})"));
            }
            if self.solver.check(&candidate, condition)? == Sat::Yes {
                return Ok(Some(candidate));
            }
        }
        Ok(None)
    }

    /// Path constraints plus the exact definitions of every opaque value.
    pub fn exact_constraints(&self, state: &State) -> Vec<String> {
        let mut exact = state.constraints.clone();
        exact.extend(state.abstractions.iter().cloned());
        exact
    }

    /// Records the taint of `count` bytes written at `pointer`.
    pub fn mark_taint(
        &self,
        state: &mut State,
        pointer: &Value,
        count: u64,
        tainted: bool,
    ) -> Result<(), String> {
        let (name, offset) = pointer.pointer()?;
        let start = crate::heap::literal(offset).and_then(|n| u64::try_from(n).ok());
        if tainted && crate::setting("TRACE_TAINT").is_some() {
            eprintln!("mark {name} {offset} {count} ({})", self.solver.context);
        }
        let Some(object) = state.memory.get_mut(name) else {
            return Ok(());
        };
        match start {
            Some(start) => {
                for byte in start..start + count {
                    if tainted {
                        object.tainted.insert(byte);
                    } else {
                        object.tainted.remove(&byte);
                    }
                }
            }
            None if tainted => object.tainted_anywhere = true,
            None => {}
        }
        Ok(())
    }

    /// Whether `count` bytes read at `pointer` may hold tainted data.
    pub fn loads_taint(&self, state: &State, pointer: &Value, count: u64) -> bool {
        let Ok((name, offset)) = pointer.pointer() else {
            return false;
        };
        let Some(object) = state.memory.get(name) else {
            return false;
        };
        if object.tainted_anywhere {
            return true;
        }
        match crate::heap::literal(offset).and_then(|n| u64::try_from(n).ok()) {
            Some(start) => object.tainted.range(start..start + count).next().is_some(),
            None => !object.tainted.is_empty(),
        }
    }
}

/// `(name value)` pairs of `names` from a `get-value` answer; compound
/// values (arrays, lets) are skipped.
fn model_values(model: &str, names: &[String]) -> Vec<(String, String)> {
    let mut values = Vec::new();
    for name in names {
        let Some((_, tail)) = model.split_once(&format!("({name} ")) else {
            continue;
        };
        let token = tail.split(')').next().unwrap_or("").trim();
        if token.starts_with("#x") || token.starts_with("#b") {
            values.push((name.clone(), token.to_owned()));
        }
    }
    values
}

/// A deterministic pseudo-random `width`-bit literal (splitmix64).
fn sampled(width: u32, sample: u64, source: u64) -> String {
    let mut state = sample.wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ source.wrapping_add(1);
    let mut next = || {
        state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    };
    let digits = width.div_ceil(4) as usize;
    let mut hex = String::new();
    while hex.len() < digits {
        hex.push_str(&format!("{:016x}", next()));
    }
    hex.truncate(digits);
    if width.is_multiple_of(4) {
        format!("#x{hex}")
    } else {
        let bits: String = hex
            .chars()
            .flat_map(|c| {
                format!("{:04b}", c.to_digit(16).unwrap_or(0))
                    .chars()
                    .collect::<Vec<_>>()
            })
            .collect();
        format!("#b{}", &bits[..width as usize])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn witness_values_are_literals_of_the_right_width() {
        assert_eq!(sampled(128, 1, 0).len(), 2 + 32);
        assert_ne!(sampled(128, 1, 0), sampled(128, 2, 0));
        assert_eq!(sampled(6, 3, 1).len(), 2 + 6);
        let model = "((input0 #x40)\n(input1 (_ bv3 8))\n(r1 #b01))";
        let names = ["input0", "input1", "r1"].map(String::from);
        assert_eq!(
            model_values(model, &names),
            vec![
                ("input0".into(), "#x40".into()),
                ("r1".into(), "#b01".into())
            ]
        );
    }
}
