//! Pointer addresses and allocation identities. Addresses are symbolic;
//! stack regions cannot alias live regions. Constants may merge with other
//! constants. A pointer choice is dereferenced only when the path resolves it.

use crate::{
    engine::{Engine, State},
    memory::Object,
    solver::Sat,
    value::{Kind, Value, and, bv, not},
};

/// Error prefix asking the engine to split the path on a condition.
pub const FORK: &str = "FORK:";

const NULL_DEREFERENCE: &str = "null pointer dereference";

/// Object of a pointer made from a symbolic integer: its address is known,
/// but it may reach no object. Null keeps the empty object name, which
/// comparisons fold as address zero; this one never folds.
pub const NO_PROVENANCE: &str = "(no provenance)";

/// A pointer with address `expr` that reaches no object.
pub fn without_provenance(expr: String, defined: String) -> Value {
    Value {
        kind: Kind::Pointer {
            object: NO_PROVENANCE.into(),
            offset: expr.clone(),
            bounded: false,
        },
        expr,
        defined,
    }
}

impl Engine {
    pub fn allocation(
        &mut self,
        state: &mut State,
        name: String,
        size: u64,
        alignment: u64,
        bytes: String,
        readonly: bool,
    ) -> Result<Value, String> {
        self.allocation_at(state, name, (size, alignment), bytes, readonly, true)
    }

    /// Construct an allocation with its actual initial liveness. Inactive
    /// stack objects may share addresses with objects that die before start.
    pub fn allocation_at(
        &mut self,
        state: &mut State,
        name: String,
        layout: (u64, u64),
        bytes: String,
        readonly: bool,
        alive: bool,
    ) -> Result<Value, String> {
        self.allocation_extent(
            state,
            name,
            (layout.0.into(), layout.1),
            bytes,
            readonly,
            alive,
        )
    }

    /// One constructor for concrete and symbolic objects; lengths remain
    /// exact in address-range and separation constraints.
    pub fn allocation_extent(
        &mut self,
        state: &mut State,
        name: String,
        layout: (crate::memory::Length, u64),
        bytes: String,
        readonly: bool,
        alive: bool,
    ) -> Result<Value, String> {
        let (size, alignment) = layout;

        if alignment == 0 || !alignment.is_power_of_two() {
            return Err("invalid allocation alignment".into());
        }
        let base = self.symbol("(_ BitVec 64)")?;
        if size.concrete().is_none() {
            self.solver.facts.symbolic_extent(&size.term());
        }
        self.solver.facts.base(&base, size.upper(), alignment);
        // The alignment constraint below holds wherever `base` can appear.
        self.simplifier.assume(
            &base,
            crate::knownbits::Known::bits(u128::from(alignment - 1), 0, 64),
        );
        let mut facts = vec![
            format!("(not (= {base} {}))", bv(0, 64)),
            format!(
                "(bvule {base} (bvsub {} {}))",
                bv(u128::from(u64::MAX), 64),
                size.term()
            ),
            format!(
                "(= (bvurem {base} {}) {})",
                bv(u128::from(alignment), 64),
                bv(0, 64)
            ),
        ];
        for object in state
            .memory
            .values()
            .filter(|o| alive && o.alive && o.allocated && (!readonly || !o.readonly || o.unique))
        {
            facts.push(format!(
                "(or (bvule (bvadd {base} {}) {}) (bvule (bvadd {} {}) {base}))",
                size.term(),
                object.base,
                object.base,
                object.size.term()
            ));
        }
        // Address-only facts, dropped from queries that see no address.
        for fact in facts {
            self.solver.facts.fact(&fact);
            state.constraints.push(fact);
        }
        state.memory.insert(
            name.clone(),
            Object {
                allocated: true,
                size,
                base: base.clone(),
                readonly,
                unique: false,
                writeonly: false,
                alignment,
                bytes,
                initialized: format!("((as const (Array (_ BitVec 64) Bool)) {readonly})"),
                poison: crate::memory::NONE_DEFINED.into(),
                known: (!readonly).then(std::collections::BTreeSet::new),
                values: std::collections::BTreeMap::new(),
                tainted: std::collections::BTreeSet::new(),
                tainted_anywhere: false,
                alive,
                stack: false,
                frame_live: true,
                managed: false,
                heap: false,
                pointers: Vec::new(),
                function: None,
            },
        );
        Ok(Value {
            expr: base,
            kind: Kind::Pointer {
                object: name,
                offset: bv(0, 64),
                bounded: true,
            },
            defined: "true".into(),
        })
    }

    /// Abbreviates every long memory array term with a solver definition.
    /// Each load reads its object's arrays once per byte (contents,
    /// initialization, poison); without names a HashMap query grew to 90 MB
    /// of text, 7.5 s of parsing alone. A `define-fun` is a macro, unlike an
    /// array equality constraint, so solving is unchanged.
    pub fn name_memory(&mut self, state: &mut State) -> Result<(), String> {
        const LIMIT: usize = 256;
        for object in state.memory.values_mut() {
            for (field, element) in [
                (&mut object.bytes, "(_ BitVec 8)"),
                (&mut object.initialized, "Bool"),
                (&mut object.poison, "Bool"),
            ] {
                if field.len() > LIMIT {
                    let name = format!("m{}", self.fresh);
                    self.fresh += 1;
                    self.solver.define(
                        &name,
                        &format!("(Array (_ BitVec 64) {element})"),
                        field,
                    )?;
                    self.simplifier.defined(&name, "Array", field);
                    *field = name;
                }
            }
        }
        Ok(())
    }

    /// The object a load, store or atomic accesses through `pointer`. A
    /// null or poison pointer there is undefined behavior (a counterexample
    /// on a feasible path); elsewhere, such as a GEP or a zero-length copy,
    /// null alone is not.
    pub fn dereference(&mut self, state: &State, pointer: &Value) -> Result<Value, String> {
        match &pointer.kind {
            Kind::Pointer { .. } | Kind::PointerChoice { .. } => {
                self.safety(
                    state,
                    &pointer.defined,
                    "null or poison pointer dereference",
                )?;
            }
            Kind::Bytes { poison, .. } => {
                self.safety(state, &not(poison), "poison pointer bytes dereferenced")?;
            }
            _ => {}
        }
        match self.resolve_pointer(state, pointer) {
            Err(error) if error == NULL_DEREFERENCE => {
                self.safety(state, "false", "null or poison pointer dereference")?;
                Err(error)
            }
            other => other,
        }
    }

    pub fn resolve_pointer(&mut self, state: &State, pointer: &Value) -> Result<Value, String> {
        match &pointer.kind {
            Kind::PointerChoice { condition, yes, no } => {
                let (when_yes, when_no) = (
                    self.solver.check(&state.constraints, condition)?,
                    self.solver.check(&state.constraints, &not(condition))?,
                );
                let chosen = match (when_yes, when_no) {
                    (_, Sat::No) => yes,
                    (Sat::No, _) => no,
                    // Both feasible: the engine splits the path on the
                    // condition and re-executes this instruction per branch.
                    (Sat::Yes, Sat::Yes) => return Err(format!("{FORK}{condition}")),
                    _ => return Err("solver unknown resolving a pointer choice".into()),
                };
                let mut value = self.resolve_pointer(state, chosen)?;
                value.defined = and(&[pointer.defined.clone(), value.defined]);
                Ok(value)
            }
            Kind::Pointer { object, .. } if object.is_empty() => Err(NULL_DEREFERENCE.into()),
            Kind::Pointer { object, .. } if object == NO_PROVENANCE => {
                Err("access through a pointer without provenance".into())
            }
            Kind::Bytes { .. } | Kind::Bits(64) => {
                Err("access through a pointer without provenance".into())
            }
            Kind::Pointer { .. } => Ok(pointer.clone()),
            _ => Err("expected pointer".into()),
        }
    }

    /// `inttoptr` of an address that is literally an allocation's base plus
    /// an offset: that base exists only in this allocation and reaches an
    /// integer only through `ptrtoint`, so the allocation was exposed and
    /// the pointer takes its provenance (Rust's exposed-provenance model;
    /// RawVec round-trips fresh allocations this way). Bounds stay checked
    /// at every access.
    pub fn exposed(&self, state: &State, address: &Value) -> Option<Value> {
        let mut expr = address.expr.clone();
        for _ in 0..4 {
            match self.simplifier.body(&expr) {
                Some(body) => expr = body.to_owned(),
                None => break,
            }
        }
        let parts: Vec<&str> = expr
            .strip_prefix("(bvadd ")
            .and_then(|e| e.strip_suffix(')'))
            .map(|e| e.splitn(2, ' ').collect())
            .unwrap_or_else(|| vec![expr.as_str()]);
        let (base, offset) = match parts.as_slice() {
            [base] => (*base, bv(0, 64)),
            [base, offset] if !base.contains('(') => (*base, (*offset).to_owned()),
            _ => return None,
        };
        let (object, _) = state
            .memory
            .iter()
            .find(|(_, o)| o.alive && o.base == base)?;
        Some(Value {
            expr: address.expr.clone(),
            kind: Kind::Pointer {
                object: object.clone(),
                offset,
                bounded: false,
            },
            defined: address.defined.clone(),
        })
    }

    /// A counterexample model: the inputs, plus the address chosen for each
    /// allocation the violated `condition` or the path depends on (at most
    /// 16). Placement is part of such a witness (an access may be misaligned
    /// only for some addresses), and a native run places its objects itself,
    /// so it may not reproduce.
    pub fn witness_model(
        &mut self,
        state: &State,
        exact: &[String],
        condition: &str,
    ) -> Result<String, String> {
        let bases = self.solver.facts.placement_bases(exact, condition);
        let omitted = bases.len().saturating_sub(16);
        let placed: Vec<(String, String, u64)> = bases
            .into_iter()
            .filter_map(|base| {
                let (name, object) = state.memory.iter().find(|(_, o)| o.base == base)?;
                Some((base, name.clone(), object.alignment))
            })
            .take(16)
            .collect();
        let mut names = self.inputs.clone();
        names.extend(placed.iter().map(|(base, ..)| base.clone()));
        let buffer_mapping = self.buffer_witness(&mut names)?;
        let mut model = self.solver.model(exact, condition, &names)?;
        model.push_str(&buffer_mapping);
        for (base, name, alignment) in placed {
            model.push_str(&format!(
                "placement: {base} = address of {name} (declared alignment {alignment}); a native run picks its own\n"
            ));
        }
        if omitted > 0 {
            model.push_str(&format!("placement: {omitted} more addresses not shown\n"));
        }
        Ok(model)
    }

    pub fn pointer_integer(&self, pointer: Value) -> Result<Value, String> {
        match &pointer.kind {
            Kind::Pointer { .. } | Kind::PointerChoice { .. } => Ok(Value {
                expr: pointer.expr.clone(),
                defined: pointer.defined.clone(),
                kind: Kind::PointerBits(Box::new(pointer)),
            }),
            // Pointer bytes read without provenance are already an integer;
            // their per-byte definedness survives the cast.
            Kind::Bytes { .. } | Kind::Bits(64) => Ok(pointer),
            _ => Err("ptrtoint requires pointer".into()),
        }
    }
}
