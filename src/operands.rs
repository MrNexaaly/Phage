//! Operand evaluation: SSA lookup, integer and pointer constants, constant
//! expressions (`inttoptr`, `ptrtoint`), poison/undef and aggregate values,
//! and materialization of computed values into fresh solver symbols.

use crate::{
    engine::{Engine, State},
    ir::{split, typed},
    value::{Kind, Value, bv, constant},
};

impl Engine {
    pub fn defined(&mut self, state: &mut State, value: &str) -> Result<String, String> {
        if matches!(value, "true" | "false") {
            return Ok(value.into());
        }
        if let Some(literal) = crate::ground::fold(value) {
            return Ok(literal);
        }
        let _ = state;
        self.definition(value, "Bool")
    }
    /// The solver name defined as `expr`: one global `define-fun` per
    /// distinct expression. A definition is an abbreviation, valid on every
    /// path, so equal computations anywhere share one term (the same hash
    /// computed in `insert` and `get`, say) and the solver compares names
    /// instead of re-deriving equalities.
    pub fn definition(&mut self, expr: &str, sort: &str) -> Result<String, String> {
        let original = expr;
        let simplified = self.simplifier.simplify(expr, sort);
        let expr = simplified.as_str();
        // Decided (a literal) or reduced to an existing name.
        if !expr.contains(['(', ' ']) || crate::heap::literal(expr).is_some() {
            return Ok(expr.to_owned());
        }
        let key = format!("{sort}|{expr}");
        if let Some(name) = self.definitions.get(&key) {
            return Ok(name.clone());
        }
        let name = format!("d{}", self.fresh);
        self.fresh += 1;
        self.solver.define(&name, sort, expr)?;
        self.definitions.insert(key, name.clone());
        self.simplifier.defined(&name, sort, expr);
        self.alias_form(original, &name);
        self.shapes.alias(original, &name);
        Ok(name)
    }
    /// `expr` as a small term: a literal (folded when ground) stays literal,
    /// anything else is bound to a fresh symbol of `sort`.
    fn name(&mut self, state: &mut State, expr: &str, sort: &str) -> Result<String, String> {
        // A literal or an existing symbol is already small; keeping the
        // symbol also keeps what is known about it (e.g. its taint).
        if crate::heap::literal(expr).is_some()
            || (!expr.is_empty() && !expr.contains(['(', ')', ' ']))
        {
            return Ok(expr.to_owned());
        }
        if let Some(literal) = crate::ground::fold(expr) {
            return Ok(literal);
        }
        let _ = state;
        self.definition(expr, sort)
    }
    pub fn materialize(&mut self, state: &mut State, value: Value) -> Result<Value, String> {
        match &value.kind {
            Kind::Aggregate(_)
            | Kind::PointerChoice { .. }
            | Kind::Undef(_)
            | Kind::UndefChoice { .. } => Ok(value),
            Kind::Float(e, s) => {
                let name = self.name(state, &value.expr, &format!("(_ BitVec {})", e + s))?;
                let defined = self.defined(state, &value.defined)?;
                Ok(Value {
                    expr: name,
                    defined,
                    ..value
                })
            }
            Kind::Bits(width) => {
                let name = self.name(state, &value.expr, &format!("(_ BitVec {width})"))?;
                let defined = self.defined(state, &value.defined)?;
                Ok(Value::bits(name, *width, defined))
            }
            // Per-byte flags stay expressions: each is small and exact.
            Kind::Bytes { flags, poison } => {
                let width = 8 * flags.len();
                let name = self.name(state, &value.expr, &format!("(_ BitVec {width})"))?;
                let mut defined = Vec::new();
                for flag in flags {
                    defined.push(self.defined(state, flag)?);
                }
                let poison = self.defined(state, poison)?;
                Ok(crate::partial::bytes(name, defined, poison))
            }
            // Equal bits by constraint, so the copied pointer stays exact.
            Kind::PointerBits(_) => {
                let name = self.name(state, &value.expr, "(_ BitVec 64)")?;
                let defined = self.defined(state, &value.defined)?;
                Ok(Value {
                    expr: name,
                    defined,
                    ..value
                })
            }
            // A null pointer has no object; its address is the constant zero.
            // A pointer without provenance keeps its address expression.
            Kind::Pointer { object, .. }
                if object.is_empty() || object == crate::pointers::NO_PROVENANCE =>
            {
                let defined = self.defined(state, &value.defined)?;
                Ok(Value { defined, ..value })
            }
            Kind::Pointer {
                object,
                offset,
                bounded,
            } => {
                let name = self.name(state, offset, "(_ BitVec 64)")?;
                let defined = self.defined(state, &value.defined)?;
                Ok(Value {
                    expr: format!(
                        "(bvadd {} {name})",
                        state.memory.get(object).ok_or("unknown allocation")?.base
                    ),
                    kind: Kind::Pointer {
                        object: object.clone(),
                        offset: name,
                        bounded: *bounded,
                    },
                    defined,
                })
            }
        }
    }
    /// An operand as used by a computation. LangRef lets each use of undef
    /// see a different value, and so every value computed from it; rather
    /// than track that variation, an observed undef is undefined bytes,
    /// exactly like uninitialized memory (partial.rs): bytes that masks or
    /// shifts fix become defined, and any other use of an undefined byte
    /// counts as not defined (stricter than LLVM; never a false proof).
    /// Branching on it is therefore undefined behavior, as LangRef says.
    pub fn operand(&mut self, state: &State, text: &str, ty: &str) -> Result<Value, String> {
        let value = self.transfer_operand(state, text, ty)?;
        observe(value)
    }
    /// An operand moved without being observed (phi, store, select arm,
    /// call argument, return, aggregate field): undef stays undef.
    pub fn transfer_operand(
        &mut self,
        state: &State,
        text: &str,
        ty: &str,
    ) -> Result<Value, String> {
        let text = text.trim();
        if text.starts_with('%') {
            return state
                .env
                .get(text)
                .cloned()
                .ok_or_else(|| format!("SSA value {text} is missing"));
        }
        if text.starts_with("ptrtoint (") {
            let inner = text
                .strip_prefix("ptrtoint (")
                .and_then(|s| s.strip_suffix(')'))
                .ok_or("invalid ptrtoint expression")?;
            let (source, target) = inner.split_once(" to ").ok_or("ptrtoint target missing")?;
            if target != "i64" || ty != "i64" {
                return Err("only 64-bit ptrtoint is supported".into());
            }
            let pointer = self.typed_operand(state, source)?;
            return self.pointer_integer(pointer);
        }
        let width = ty.strip_prefix('i').and_then(|w| w.parse::<u32>().ok());
        if let Some((count, lane)) = crate::vectors::vector_type(ty) {
            return crate::vectors::constant(text, count, lane);
        }
        if ty.starts_with(['{', '[', '<']) {
            return match text {
                "poison" => crate::value::aggregate(ty, false),
                "zeroinitializer" => crate::value::aggregate(ty, true),
                _ => Err(format!("unsupported aggregate operand {text}")),
            };
        }
        if let Some(format) = crate::floats::format(ty)
            && !matches!(text, "poison" | "undef")
        {
            self.floats_on()?;
            let expr = crate::floats::constant(text, format)?;
            return Ok(Value {
                expr,
                kind: Kind::Float(format.0, format.1),
                defined: "true".into(),
            });
        }
        if ty == "ptr" && text == "undef" {
            // Keep pointer undef distinct from poison until it is observed.
            // This lets an absorbing Boolean operation discard an undef
            // comparison, as Rust's BTreeMap does for an unused niche field.
            return Ok(Value {
                expr: self.symbol("(_ BitVec 64)")?,
                kind: Kind::Undef(64),
                defined: "true".into(),
            });
        }
        if ty == "ptr" && text == "poison" {
            return Ok(Value {
                expr: crate::value::bv(0, 64),
                kind: Kind::Pointer {
                    object: String::new(),
                    offset: crate::value::bv(0, 64),
                    bounded: false,
                },
                defined: "false".into(),
            });
        }
        if matches!(text, "poison" | "undef") {
            let width = width.ok_or("undefined non-integer value is unsupported")?;
            let expr = self.symbol(&format!("(_ BitVec {width})"))?;
            if text == "undef" {
                // LLVM LangRef: each use of undef may choose a different value.
                // Keep unused undef bindings, but refuse to treat an observed
                // use as one stable symbolic input (freeze is unsupported).
                return Ok(Value {
                    expr,
                    kind: Kind::Undef(width),
                    defined: "true".into(),
                });
            }
            return Ok(Value::bits(
                expr,
                width,
                if text == "poison" {
                    "false".into()
                } else {
                    "true".into()
                },
            ));
        }
        if ty == "ptr" {
            if text.starts_with("getelementptr ") {
                let (symbol, addend, inbounds) = crate::constants::pointer_target(text)
                    .ok_or("unsupported constant getelementptr operand")?;
                let key = Self::qualify(&state.module, &symbol);
                let key = self.global_keys.get(&key).cloned().unwrap_or(key);
                let object = state
                    .memory
                    .get(&key)
                    .ok_or("constant GEP target missing")?;
                let offset = crate::value::bv(u128::from(addend), 64);
                let defined = (!inbounds || addend <= object.size.upper()).to_string();
                return Ok(Value {
                    expr: format!("(bvadd {} {offset})", object.base),
                    kind: Kind::Pointer {
                        object: key,
                        offset,
                        bounded: inbounds,
                    },
                    defined,
                });
            }
            if let Some(address) = text
                .strip_prefix("inttoptr (i64 ")
                .and_then(|t| t.split_whitespace().next())
                .and_then(|n| n.parse::<i64>().ok())
                .map(|n| n as u64)
            {
                let name = format!("@int.{address}");
                if address != 0 && !state.memory.contains_key(&name) {
                    return Err("constant pointer object missing".into());
                }
                return Ok(Value {
                    expr: crate::value::bv(u128::from(address), 64),
                    kind: Kind::Pointer {
                        object: if address == 0 { String::new() } else { name },
                        offset: crate::value::bv(0, 64),
                        bounded: address != 0,
                    },
                    defined: "true".into(),
                });
            }
            if text == "null" {
                return Ok(Value {
                    expr: crate::value::bv(0, 64),
                    kind: Kind::Pointer {
                        object: String::new(),
                        offset: crate::value::bv(0, 64),
                        bounded: false,
                    },
                    defined: "true".into(),
                });
            }
            let key = Self::qualify(&state.module, text);
            let key = self.global_keys.get(&key).cloned().unwrap_or(key);
            if let Some(object) = state.memory.get(&key) {
                return Ok(Value {
                    expr: object.base.clone(),
                    kind: Kind::Pointer {
                        object: key,
                        offset: crate::value::bv(0, 64),
                        bounded: true,
                    },
                    defined: "true".into(),
                });
            }
            return Err(format!("unsupported pointer operand {text}"));
        }
        constant(text, width.ok_or_else(|| format!("unsupported type {ty}"))?)
    }
    pub fn typed_operand(&mut self, state: &State, text: &str) -> Result<Value, String> {
        let value = self.typed_transfer(state, text)?;
        observe(value)
    }
    pub fn typed_transfer(&mut self, state: &State, text: &str) -> Result<Value, String> {
        let (ty, operand) = typed(text)?;
        // Constant expressions contain spaces; attributes precede them.
        let operand = match ["inttoptr (", "ptrtoint (", "getelementptr ", "splat (", "<"]
            .iter()
            .find_map(|prefix| operand.find(prefix))
        {
            Some(start) => operand[start..].trim(),
            None => operand.split_whitespace().last().ok_or("operand missing")?,
        };
        self.transfer_operand(state, operand, ty)
    }

    /// Comparison with a wholly undef pointer or integer operand is an undef
    /// i1, not poison (LangRef: an undef operand may take any value, so the
    /// result may be either Boolean). Keep it lazy so a following absorbing
    /// `or true` / `and false` can define the result; rustc emits exactly this
    /// for `Option<&T>::copied().filter(..)` on the `None` path. Other sub-byte
    /// operations remain unsupported; direct observation retains the existing
    /// undef behavior. A poison peer is not silently discarded, and a partly
    /// undefined value (bytes or a choice) keeps the strict path.
    pub fn undef_compare(
        &mut self,
        same_sign: bool,
        width: u32,
        left: &Value,
        right: &Value,
    ) -> Result<Option<Value>, String> {
        let undef = |v: &Value| matches!(v.kind, Kind::Undef(w) if w == width);
        if !undef(left) && !undef(right) {
            return Ok(None);
        }
        if same_sign {
            return Err("samesign comparison with an undef operand is unsupported".into());
        }
        if left.defined != "true" || right.defined != "true" {
            return Err("comparison mixes an undef and poison operand".into());
        }
        Ok(Some(Value {
            expr: self.symbol("(_ BitVec 1)")?,
            kind: Kind::Undef(1),
            defined: "true".into(),
        }))
    }

    /// Parses and evaluates an integer or pointer comparison. Pointer undef
    /// stays lazy until a later absorbing Boolean operation can discard it.
    pub fn compare_instruction(&mut self, state: &State, rest: &str) -> Result<Value, String> {
        let args = split(rest);
        if args.len() != 2 {
            return Err("comparison operands missing".into());
        }
        let words: Vec<_> = args[0].split_whitespace().collect();
        let same = words.first() == Some(&"samesign");
        let start = usize::from(same);
        let predicate = words.get(start).ok_or("comparison predicate missing")?;
        let ty = words.get(start + 1).ok_or("comparison type missing")?;
        let left = words
            .get(start + 2..)
            .filter(|w| !w.is_empty())
            .ok_or("comparison value missing")?
            .join(" ");
        let left = self.transfer_operand(state, &left, ty)?;
        let right = self.transfer_operand(state, args[1], ty)?;
        let width = if *ty == "ptr" {
            Some(64)
        } else {
            ty.strip_prefix('i').and_then(|w| w.parse::<u32>().ok())
        };
        let undef = match width {
            Some(width) => self.undef_compare(same, width, &left, &right)?,
            None => None,
        };
        match undef {
            Some(result) => Ok(result),
            None => crate::value::compare(predicate, same, &observe(left)?, &observe(right)?),
        }
    }

    /// Exact absorbing identities for a sub-byte undef. Other operations on
    /// it stay unknown because Phage does not otherwise track bit-level undef.
    pub fn undef_bitwise(
        &mut self,
        state: &State,
        op: &str,
        flags: &[&str],
        left: &Value,
        right: &Value,
    ) -> Result<Option<Value>, String> {
        let undef = |value: &Value| matches!(value.kind, Kind::Undef(1));
        if !undef(left) && !undef(right) {
            return Ok(None);
        }
        if !flags.is_empty() || !matches!(op, "and" | "or") {
            return Err("operation on a sub-byte LLVM undef is unsupported".into());
        }
        let other = match (undef(left), undef(right)) {
            (true, false) => right,
            (false, true) => left,
            // A bitwise combination of two undef bits is again any value at each
            // use: keep it lazy so a later absorbing operation can define it.
            _ if matches!(op, "and" | "or" | "xor") => {
                return Ok(Some(Value {
                    expr: self.symbol("(_ BitVec 1)")?,
                    kind: Kind::Undef(1),
                    defined: "true".into(),
                }));
            }
            _ => return Err("operation on two sub-byte LLVM undefs is unsupported".into()),
        };
        if other.defined != "true" {
            return Err("bitwise operation mixes undef and poison".into());
        }
        let fixed = self.concrete(state, &other.expr)?;
        match (op, fixed) {
            ("or", Some(1)) => Ok(Some(Value::bits(bv(1, 1), 1, "true".into()))),
            ("and", Some(0)) => Ok(Some(Value::bits(bv(0, 1), 1, "true".into()))),
            _ => Err("unmasked sub-byte LLVM undef is unsupported".into()),
        }
    }
}

/// An undef (or undef choice) as undefined bytes; other values unchanged.
/// This is also exactly what storing it leaves in memory.
pub fn observe(value: Value) -> Result<Value, String> {
    match value.kind {
        Kind::Undef(width) if width.is_multiple_of(8) => {
            Ok(crate::partial::undef((width / 8) as usize))
        }
        Kind::Undef(width) => Ok(Value::bits(
            crate::value::bv(0, width),
            width,
            "false".into(),
        )),
        // Undefined bytes where the choice picks undef.
        Kind::UndefChoice { condition, other } => {
            let keep = crate::value::not(&condition);
            let width = other.width()?;
            if !width.is_multiple_of(8) {
                let defined = crate::value::and(&[keep, other.defined.clone()]);
                return Ok(Value::bits(other.expr, width, defined));
            }
            let (flags, poison) = crate::partial::store_flags(&other, (width / 8) as usize)?;
            let flags = flags
                .iter()
                .map(|f| crate::value::and(&[keep.clone(), f.clone()]))
                .collect();
            Ok(crate::partial::bytes(
                other.expr,
                flags,
                crate::value::and(&[keep, poison]),
            ))
        }
        _ => Ok(value),
    }
}
