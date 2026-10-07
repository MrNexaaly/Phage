//! Call instructions: callee parsing (direct and through code-object
//! pointers), scoped `noreturn` divergence, parameter and result contracts,
//! intrinsic / allocator / byte-operation models and frames for defined
//! functions.

use crate::{
    engine::{Engine, State, Step},
    ir::{split, typed},
    memory,
    symbols::matching_paren,
    value::{self, Kind, Value, and, bv, truth},
};

impl Engine {
    pub fn call(
        &mut self,
        state: &mut State,
        opcode: &str,
        rest: &str,
        destination: Option<&str>,
    ) -> Result<Step, String> {
        let call = if opcode == "tail" {
            rest.strip_prefix("call ")
                .ok_or("unsupported tail instruction")?
        } else {
            rest
        };
        let (at, open) = callee_span(call).ok_or("call callee missing")?;
        let close = matching_paren(call, open).ok_or("call operands unterminated")?;
        let suffix = call[close + 1..].trim();
        let (attributes, bundles) = suffix
            .split_once('[')
            .map_or((suffix, None), |(a, b)| (a, Some(b)));
        if !attributes.split_whitespace().all(|word| {
            word.strip_prefix('#')
                .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        }) {
            return Err("call operand bundles are unsupported".into());
        }
        let args = split(&call[open + 1..close]);
        // An indirect call dispatches on the function a code-object
        // pointer denotes, resolved in that function's own module.
        let (from, callee) = if call[at..].starts_with('%') {
            let pointer = self.operand(state, &call[at..open], "ptr")?;
            self.safety(state, &pointer.defined, "poison function pointer called")?;
            let pointer = self.resolve_pointer(state, &pointer)?;
            let (object, offset) = pointer.pointer()?;
            let target = state
                .memory
                .get(object)
                .and_then(|o| o.function.clone())
                .ok_or("indirect call through a non-function pointer")?;
            self.safety(
                state,
                &format!("(= {offset} {})", bv(0, 64)),
                "indirect call through a displaced function pointer",
            )?;
            target
        } else {
            let name = call[at + 1..open].trim_matches('"').to_owned();
            (state.module.clone(), name)
        };
        let callee = callee.as_str();
        if bundles.is_some() && callee != "llvm.assume" {
            return Err("call operand bundles are unsupported".into());
        }
        let definition = if callee.starts_with("llvm.") {
            None
        } else {
            self.function(callee, &from)?
        };
        // Artifact definitions always execute, including panic-shaped names.
        let artifact_definition = definition.as_ref().is_some_and(|f| {
            self.modules
                .iter()
                .any(|m| m.name == f.module && m.name == "user" && m.defines(callee))
        });
        if !artifact_definition
            && crate::symbols::is_panic(callee)
            && definition.as_ref().is_some_and(|f| f.module != "user")
            && matches!(self.library, crate::engine::Library::Open(_))
        {
            return Ok(Step::Panic(format!("Rust panic call {callee}")));
        }
        if self.diverges(callee, &from)? {
            return Ok(Step::Panic(format!(
                "call never returns (panic, abort, exit or non-termination): {callee}"
            )));
        }
        self.call_contracts(state, callee, &args, definition.as_deref())?;
        if callee == "llvm.experimental.noalias.scope.decl" {
            // This intrinsic declares optimizer alias metadata; it
            // has no executed memory access. Object identity and
            // bounds remain checked by the memory model.
            return Ok(Step::Continue);
        }
        if callee.starts_with("llvm.lifetime.") {
            return self.lifetime_call(state, callee, &args);
        }
        if callee == "llvm.assume" {
            if args.len() != 1 || typed(args[0])?.0 != "i1" {
                return Err(format!(
                    "unsupported intrinsic signature {callee}: {args:?}"
                ));
            }
            if let Some(bundles) = bundles {
                if args[0] != "i1 true" {
                    return Err(
                        "unsupported llvm.assume bundle condition (requires i1 true)".into(),
                    );
                }
                self.assume_bundles(state, bundles)?;
            }
            // LangRef: a false or poison assumption is immediate UB,
            // so it is an obligation, never a path fact to trust.
            let condition =
                self.typed_operand(state, args.first().ok_or("assume operand missing")?)?;
            let holds = and(&[condition.defined.clone(), truth(&condition)?]);
            self.safety(
                state,
                &holds,
                "llvm.assume can be false (undefined behavior)",
            )?;
            return Ok(Step::Continue);
        }
        if callee.starts_with("llvm.is.constant.") {
            // A folding hint still present in the artifact: code generation
            // lowers it to false, which is what the compiled program runs.
            let answer = Value::bits(crate::value::bv(0, 1), 1, "true".into());
            return self.finish_call(state, destination, Some(answer));
        }
        let result_attributes = call[..at].to_owned();
        let noundef_result = result_attributes.split_whitespace().any(|w| w == "noundef");
        let math_flags: Vec<&str> = result_attributes
            .split_whitespace()
            .filter(|w| {
                matches!(
                    *w,
                    "nnan" | "ninf" | "nsz" | "arcp" | "contract" | "afn" | "reassoc" | "fast"
                )
            })
            .collect();
        let modeled = if callee.starts_with("llvm.") && callee.contains(".with.overflow.") {
            if args.len() != 2 {
                return Err(format!(
                    "unsupported intrinsic signature {callee}: {args:?}"
                ));
            }
            let left = self.typed_operand(state, args[0])?;
            let right = self.typed_operand(state, args[1])?;
            Some(Some(self.overflow_intrinsic(callee, &left, &right)?))
        } else if callee.starts_with("llvm.threadlocal.address.") {
            // One thread executes the property, so a thread-local's
            // address for this thread is the global itself.
            let pointer =
                self.typed_operand(state, args.first().ok_or("threadlocal operand missing")?)?;
            pointer.pointer()?;
            Some(Some(pointer))
        } else if let Some(value) = self.float_intrinsic(state, callee, &args, &math_flags)? {
            Some(Some(value))
        } else if let Some(value) = self.intrinsic(state, callee, &args)? {
            // A rotate mixing random-derived bits is opaque (opaque.rs).
            let rotate = callee.starts_with("llvm.fsh") && !self.tainted.is_empty();
            let value = if rotate {
                let mut operands = Vec::new();
                for arg in args.iter().filter(|a| !a.is_empty()) {
                    operands.push(self.typed_operand(state, arg)?);
                }
                self.opaque_rotate(state, callee, &operands, &value)?
                    .unwrap_or(value)
            } else {
                value
            };
            Some(Some(value))
        } else if let Some(result) = self.allocator(state, callee, &args)? {
            Some(result)
        } else if definition.is_none()
            && let Some(result) = self.environment_call(state, callee, &args)?
        {
            Some(result)
        } else {
            self.bytes_call(state, callee, &args)?
        };
        if let Some(result) = modeled {
            let result = result
                .map(|v| result_contract(&result_attributes, v))
                .transpose()?;
            if noundef_result && let Some(value) = &result {
                self.safety(
                    state,
                    &value::fully_defined(value),
                    "poison result of a noundef call",
                )?;
            }
            return self.finish_call(state, destination, result);
        }
        if let Some(function) = definition {
            let mut values = Vec::new();
            for arg in args.iter().filter(|a| !a.is_empty()) {
                values.push(self.typed_transfer(state, arg)?);
            }
            Ok(Step::Call(
                function,
                values,
                destination.map(str::to_owned),
                result_attributes,
            ))
        } else if let crate::engine::Library::Failed(reason) = &self.library {
            Err(format!("unmodeled call {callee} (library: {reason})"))
        } else {
            Err(format!("unmodeled call {callee}"))
        }
    }

    /// Parameter attributes whose violation is immediate UB, from the call
    /// site and the callee's definition: `noundef` (no poison in any leaf)
    /// and `dereferenceable(N)` (N accessible bytes).
    fn call_contracts(
        &mut self,
        state: &State,
        callee: &str,
        args: &[&str],
        definition: Option<&crate::ir::Function>,
    ) -> Result<(), String> {
        for (index, arg) in args.iter().filter(|a| !a.is_empty()).enumerate() {
            let (ty, attributes) = typed(arg)?;
            let mut text = attributes.to_owned();
            if let Some(declared) = definition.and_then(|d| d.parameter_attributes.get(index)) {
                text = format!("{text} {declared}");
            }
            let words: Vec<&str> = text.split_whitespace().collect();
            let noundef = words.contains(&"noundef");
            let dereferenceable = words
                .iter()
                .filter_map(|w| {
                    w.strip_prefix("dereferenceable(")
                        .and_then(|n| n.strip_suffix(')'))
                        .and_then(|n| n.parse::<u64>().ok())
                })
                .max()
                .filter(|b| *b > 0);
            let facts = !words
                .iter()
                .any(|w| *w == "nonnull" || *w == "align" || w.starts_with("range("));
            if !noundef && dereferenceable.is_none() && facts {
                continue;
            }
            let value = self.typed_transfer(state, arg)?;
            // LangRef: an argument violating nonnull, align or range is
            // poison; with noundef that is undefined behavior, including
            // zero-length byte intrinsics.
            // For a concrete nonzero byte operation, range() observes the
            // pointer and its alignment: a violated align is poison and UB.
            // Zero/possibly-zero lengths must retain the attribute check here.
            let observed_align = ["llvm.memcpy.", "llvm.memmove.", "llvm.memset."]
                .iter()
                .any(|prefix| callee.starts_with(prefix))
                && args
                    .get(2)
                    .map(|arg| self.typed_transfer(state, arg))
                    .transpose()?
                    .is_some_and(|v| crate::heap::literal(&v.expr).is_some_and(|n| n > 0));
            let holds = attribute_facts(ty, &text, &value, observed_align)?;
            if noundef {
                self.safety(
                    state,
                    &and(&[value::fully_defined(&value), holds]),
                    "argument is poison or violates its noundef attributes",
                )?;
            } else if holds != "true"
                && self.solver.check(
                    &state.constraints,
                    &and(&[value::fully_defined(&value), value::not(&holds)]),
                )? != crate::solver::Sat::No
            {
                // The retained artifact is the proof boundary, including
                // optimizer-inferred attributes on intrinsics. We cannot
                // discard an attribute and still claim to have checked it.
                // Until argument attribute poison is threaded into all call
                // models, stop this path as unknown (not immediate UB).
                return Err(format!(
                    "a defined argument of {callee} may violate its nonnull/align/range \
                     attribute, which makes it poison in LLVM; argument attribute poison \
                     propagation is unsupported"
                ));
            }
            if let Some(bytes) = dereferenceable {
                let pointer = self.resolve_pointer(state, &value)?;
                let valid = memory::access(&state.memory, &pointer, bytes, 1)?;
                self.safety(state, &valid, "non-dereferenceable pointer argument")?;
            }
        }
        Ok(())
    }

    /// Binds a modeled call's result and continues after the call.
    fn finish_call(
        &mut self,
        state: &mut State,
        destination: Option<&str>,
        result: Option<Value>,
    ) -> Result<Step, String> {
        match (destination, result) {
            (Some(name), Some(value)) => {
                let opaque = value.expr.starts_with("(ru_");
                let value = self.materialize(state, value)?;
                if opaque {
                    self.taint(&value);
                }
                state.env.insert(name.to_owned(), value);
            }
            (Some(_), None) => return Err("void call result is used".into()),
            _ => {}
        }
        Ok(Step::Continue)
    }

    fn overflow_intrinsic(&mut self, name: &str, a: &Value, b: &Value) -> Result<Value, String> {
        let width = a.width()?;
        let op = if name.contains("add.") {
            "add"
        } else if name.contains("sub.") {
            "sub"
        } else if name.contains("mul.") {
            "mul"
        } else {
            return Err("unknown overflow intrinsic".into());
        };
        let signed = name.starts_with("llvm.s");
        let flag = if signed { "nsw" } else { "nuw" };
        let checked = value::binary(op, &[flag], a, b)?;
        let arithmetic = value::binary(op, &[], a, b)?;
        let valid = and(&[a.defined.clone(), b.defined.clone()]);
        let overflow = Value::bits(
            format!("(ite {} {} {})", checked.defined, bv(0, 1), bv(1, 1)),
            1,
            valid.clone(),
        );
        Ok(Value {
            expr: String::new(),
            kind: Kind::Aggregate(vec![
                Value::bits(arithmetic.expr, width, valid.clone()),
                overflow,
            ]),
            defined: valid,
        })
    }
}

/// Start of the callee (`@f` or `%p`) and its argument list's parenthesis.
/// Return types and attributes may contain parentheses of their own.
pub(crate) fn callee_span(call: &str) -> Option<(usize, usize)> {
    let bytes = call.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if matches!(bytes[i], b'@' | b'%') {
            let start = i;
            i += 1;
            if bytes.get(i) == Some(&b'"') {
                i += 1 + call[i + 1..].find('"')? + 1;
            } else {
                while i < bytes.len()
                    && (bytes[i].is_ascii_alphanumeric() || b"_.$-".contains(&bytes[i]))
                {
                    i += 1;
                }
            }
            if bytes.get(i) == Some(&b'(') {
                return Some((start, i));
            }
        } else {
            i += 1;
        }
    }
    None
}

/// `value` returned under `attributes`: violated `nonnull`, `align` or
/// `range` facts make it poison (the caller checks `noundef`).
pub fn result_contract(attributes: &str, value: Value) -> Result<Value, String> {
    let ty = match value.kind {
        Kind::Pointer { .. } | Kind::PointerChoice { .. } => "ptr".to_owned(),
        _ => match value.width() {
            Ok(width) => format!("i{width}"),
            Err(_) => String::new(),
        },
    };
    let holds = attribute_facts(&ty, attributes, &value, false)?;
    Ok(crate::partial::restrict(value, &holds))
}

/// Conditions under which the `nonnull`, `align N` (unless `skip_align`)
/// and `range(iN a, b)` facts in `attributes` hold for `value`, conjoined.
fn attribute_facts(
    ty: &str,
    attributes: &str,
    value: &Value,
    skip_align: bool,
) -> Result<String, String> {
    let words: Vec<&str> = attributes.split_whitespace().collect();
    let asserts = words.contains(&"nonnull")
        || (!skip_align && words.contains(&"align"))
        || attributes.contains("range(");
    if asserts && value.expr.is_empty() {
        return Err("value attributes on an aggregate are unsupported".into());
    }
    let mut facts = Vec::new();
    if words.contains(&"nonnull") {
        facts.push(format!("(not (= {} {}))", value.expr, bv(0, 64)));
    }
    if let Some(alignment) = words
        .windows(2)
        .filter(|w| w[0] == "align" && !skip_align)
        .filter_map(|w| w[1].parse::<u64>().ok())
        .max()
        .filter(|a| *a > 1)
    {
        facts.push(format!(
            "(= (bvurem {} {}) {})",
            value.expr,
            bv(u128::from(alignment), 64),
            bv(0, 64)
        ));
    }
    let mut rest = attributes;
    while let Some(start) = rest.find("range(") {
        let open = start + "range".len();
        let close = matching_paren(rest, open).ok_or("range attribute unterminated")?;
        let inner = &rest[open + 1..close];
        rest = &rest[close + 1..];
        let (bounds_ty, bounds) = inner.split_once(' ').ok_or("range attribute malformed")?;
        if bounds_ty != ty {
            return Err("range attribute type differs".into());
        }
        let width = value.width()?;
        let (low, high) = bounds.split_once(',').ok_or("range attribute malformed")?;
        let low = value::constant(low.trim(), width)?.expr;
        let high = value::constant(high.trim(), width)?.expr;
        let v = &value.expr;
        // Half-open and wrapping: [low, high) modulo 2^width.
        facts.push(format!(
            "(ite (bvule {low} {high}) (and (bvuge {v} {low}) (bvult {v} {high})) (or (bvuge {v} {low}) (bvult {v} {high})))"
        ));
    }
    Ok(and(&facts))
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod lifetime_tests;
