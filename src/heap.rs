//! Heap objects behind Rust's stable allocator ABI (`__rust_alloc` and
//! friends), pointer values stored in memory, byte copies and comparisons.
//! Allocation is assumed to succeed, as in Kani; every other contract of
//! the ABI (layout on free, use after free, zero size) is an obligation.

use crate::{
    engine::{Engine, State},
    memory::{self, Object, PointerSlot},
    solver::Sat,
    value::{Kind, Value, and, bv},
};
use std::collections::BTreeMap;

mod alloc;

impl Engine {
    /// The value of `expr` when the path constraints fix it, else `None`.
    pub fn concrete(&mut self, state: &State, expr: &str) -> Result<Option<u128>, String> {
        if let Some(n) = literal(expr) {
            return Ok(Some(n));
        }
        let Some(value) = self.solver.value(&state.constraints, expr)? else {
            return Ok(None);
        };
        let fixed = self
            .solver
            .check(&state.constraints, &format!("(not (= {expr} {value}))"))?;
        Ok((fixed == Sat::No).then(|| literal(&value)).flatten())
    }

    /// Stores a pointer: its address bytes plus a provenance shadow entry.
    pub fn store_pointer(
        &mut self,
        state: &mut State,
        target: &Value,
        value: &Value,
    ) -> Result<(), String> {
        let (name, offset) = target.pointer()?;
        let name = name.to_owned();
        let offset = self
            .concrete(state, offset)?
            .map(|n| bv(n, 64))
            .unwrap_or_else(|| offset.to_owned());
        let address = Value::bits(value.expr.clone(), 64, value.defined.clone());
        memory::store(&mut state.memory, target, &address)?;
        let object = state.memory.get_mut(&name).ok_or("unknown allocation")?;
        object.forget_pointers(&offset, &bv(8, 64));
        object.pointers.push(PointerSlot {
            offset,
            value: value.clone(),
            valid: "true".into(),
        });
        Ok(())
    }

    /// Loads a pointer. Provenance comes from the shadow entry, which is
    /// used only if the bytes provably still hold that exact address;
    /// otherwise the pointer keeps its address without provenance.
    pub fn load_pointer(&mut self, state: &State, source: &Value) -> Result<Value, String> {
        let loaded = memory::load(&state.memory, source, 64)?;
        let Some(stored) = self.pointer_slot(state, source)? else {
            // Bytes none of which can be defined read as a poison pointer.
            let some_defined = match &loaded.kind {
                Kind::Bytes { flags, poison } => {
                    and(&[crate::value::not(poison), crate::value::or(flags)])
                }
                _ => loaded.defined.clone(),
            };
            if self.solver.check(&state.constraints, &some_defined)? == Sat::No {
                return Ok(Value {
                    expr: bv(0, 64),
                    kind: Kind::Pointer {
                        object: String::new(),
                        offset: bv(0, 64),
                        bounded: false,
                    },
                    defined: "false".into(),
                });
            }
            // No provenance: only a provable null is a usable pointer.
            if self.solver.check(
                &state.constraints,
                &format!("(not (= {} {}))", loaded.expr, bv(0, 64)),
            )? == Sat::No
            {
                return Ok(Value {
                    expr: bv(0, 64),
                    kind: Kind::Pointer {
                        object: String::new(),
                        offset: bv(0, 64),
                        bounded: false,
                    },
                    defined: loaded.defined,
                });
            }
            // Integer bytes read as a pointer (a speculative load of an enum
            // slot, say): the address and per-byte definedness are exact, no
            // object is reachable through it.
            return Ok(loaded);
        };
        let intact = self.solver.check(
            &state.constraints,
            &format!("(not (= {} {}))", loaded.expr, stored.expr),
        )?;
        if intact == Sat::Unknown {
            return Err("solver unknown checking stored pointer bytes".into());
        }
        if intact != Sat::No {
            // Bytes that may no longer be the stored pointer keep only
            // their address.
            return Ok(loaded);
        }
        Ok(Value {
            defined: and(&[stored.defined.clone(), loaded.defined]),
            ..stored
        })
    }

    /// The stored pointer whose exact bytes an `i64` load at `source` read,
    /// if any: the loaded integer is then a copy that carries provenance.
    pub fn copied_pointer(
        &mut self,
        state: &State,
        source: &Value,
        loaded: &Value,
    ) -> Result<Option<Value>, String> {
        let Some(stored) = self.pointer_slot(state, source)? else {
            return Ok(None);
        };
        let intact = self.solver.check(
            &state.constraints,
            &format!("(not (= {} {}))", loaded.expr, stored.expr),
        )?;
        if intact == Sat::Unknown {
            return Err("solver unknown checking copied pointer bytes".into());
        }
        Ok((intact == Sat::No).then(|| Value {
            defined: and(&[stored.defined.clone(), loaded.defined.clone()]),
            ..stored
        }))
    }

    /// Resolve a symbolic read against this object's finite provenance
    /// shadow. A possible slot splits the path before the instruction is
    /// retried, so its byte-integrity check uses that slot's path
    /// constraints. The residual path keeps the exact bytes without gaining
    /// provenance; equal addresses in other slots or objects do not qualify.
    /// Both pointer loads and whole i64 copies use this same resolution.
    fn pointer_slot(&mut self, state: &State, source: &Value) -> Result<Option<Value>, String> {
        let (name, offset) = source.pointer()?;
        let object = state.memory.get(name).ok_or("unknown allocation")?;
        if object.pointers.is_empty() {
            return Ok(None);
        }
        let offset = self
            .concrete(state, offset)?
            .map(|n| bv(n, 64))
            .unwrap_or_else(|| offset.to_owned());
        for slot in &object.pointers {
            if let (Some(read), Some(stored)) = (literal(&offset), literal(&slot.offset))
                && read != stored
            {
                continue;
            }
            let condition = and(&[slot.valid.clone(), format!("(= {offset} {})", slot.offset)]);
            let condition = self.simplifier.simplify(&condition, "Bool");
            if condition == "false" {
                continue;
            }
            if condition == "true" {
                return Ok(Some(slot.value.clone()));
            }
            match self.solver.check(&state.constraints, &condition)? {
                Sat::No => continue,
                Sat::Unknown => return Err("solver unknown resolving a pointer slot".into()),
                Sat::Yes => {}
            }
            match self
                .solver
                .check(&state.constraints, &crate::value::not(&condition))?
            {
                Sat::No => return Ok(Some(slot.value.clone())),
                Sat::Yes => return Err(format!("{}{condition}", crate::pointers::FORK)),
                Sat::Unknown => return Err("solver unknown resolving a pointer slot".into()),
            }
        }
        Ok(None)
    }

    /// A pointer made from a known integer address: a zero-size object at
    /// that address (Rust's dangling `NonNull`), or null for zero.
    pub fn dangling(&mut self, state: &mut State, address: u64) -> Value {
        if address == 0 {
            return Value {
                expr: bv(0, 64),
                kind: Kind::Pointer {
                    object: String::new(),
                    offset: bv(0, 64),
                    bounded: false,
                },
                defined: "true".into(),
            };
        }
        let name = format!("@int.{address}");
        if !state.memory.contains_key(&name) {
            self.solver.facts.obstacle();
        }
        state.memory.entry(name.clone()).or_insert_with(|| Object {
            allocated: false,
            size: 0.into(),
            base: bv(u128::from(address), 64),
            readonly: true,
            unique: false,
            writeonly: false,
            alignment: 1 << address.trailing_zeros().min(12),
            bytes: format!(
                "((as const (Array (_ BitVec 64) (_ BitVec 8))) {})",
                bv(0, 8)
            ),
            initialized: "((as const (Array (_ BitVec 64) Bool)) false)".into(),
            poison: crate::memory::NONE_DEFINED.into(),
            known: None,
            values: BTreeMap::new(),
            tainted: std::collections::BTreeSet::new(),
            tainted_anywhere: false,
            alive: true,
            stack: false,
            frame_live: true,
            managed: false,
            heap: false,
            pointers: Vec::new(),
            function: None,
        });
        Value {
            expr: bv(u128::from(address), 64),
            kind: Kind::Pointer {
                object: name,
                offset: bv(0, 64),
                bounded: true,
            },
            defined: "true".into(),
        }
    }

    /// Creates dangling objects for `inttoptr (i64 N to ptr)` constants.
    pub fn constant_pointers(&mut self, state: &mut State, code: &str) {
        for part in code.split("inttoptr (i64 ").skip(1) {
            if let Some(address) = part
                .split_whitespace()
                .next()
                .and_then(|n| n.parse::<i64>().ok())
                .map(|n| n as u64)
            {
                self.dangling(state, address);
            }
        }
    }
}

pub(crate) fn literal(expr: &str) -> Option<u128> {
    let expr = expr.trim();
    if let Some(rest) = expr.strip_prefix("(_ bv") {
        // `(_ bvN W)` is N modulo 2^W.
        let mut words = rest.split_whitespace();
        let value: u128 = words.next()?.parse().ok()?;
        let width: u32 = words.next()?.trim_end_matches(')').parse().ok()?;
        return Some(if width >= 128 {
            value
        } else {
            value & ((1u128 << width) - 1)
        });
    }
    if let Some(hex) = expr.strip_prefix("#x") {
        return u128::from_str_radix(hex, 16).ok();
    }
    if let Some(bits) = expr.strip_prefix("#b") {
        return u128::from_str_radix(bits, 2).ok();
    }
    None
}

/// Allocator entry points by exact name or v0 path `__rustc::<name>`.
fn allocator_kind(symbol: &str) -> Option<&'static str> {
    const NAMES: [&str; 5] = [
        "__rust_alloc",
        "__rust_alloc_zeroed",
        "__rust_dealloc",
        "__rust_realloc",
        "__rust_no_alloc_shim_is_unstable_v2",
    ];
    let name = match crate::symbols::v0_segments(symbol) {
        Some(path) if path.len() == 2 && path[0] == "__rustc" => path[1],
        _ => symbol,
    };
    NAMES.into_iter().find(|n| *n == name)
}

#[cfg(test)]
mod provenance_tests;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod symbolic_tests;
