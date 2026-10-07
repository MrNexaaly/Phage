//! Byte operations: llvm.memcpy / memmove / memset (concrete lengths byte
//! by byte, symbolic lengths over small objects), and libc bcmp / memcmp
//! with only their specified results. Call-site `align` promises and
//! overlap rules are obligations; pointer provenance moves with whole slots.

use crate::{
    engine::{Engine, State},
    heap::literal,
    value::{Value, and, bv},
};

/// Largest byte range modeled by one copy or comparison.
const COPY_LIMIT: u64 = 4096;
/// Largest object scanned byte by byte for a symbolic copy length.
const SYMBOLIC_COPY_LIMIT: u64 = 256;

impl Engine {
    /// llvm.memcpy / llvm.memmove / llvm.memset and libc bcmp / memcmp.
    pub fn bytes_call(
        &mut self,
        state: &mut State,
        callee: &str,
        args: &[&str],
    ) -> Result<Option<Option<Value>>, String> {
        let kind = if callee.starts_with("llvm.memcpy.") {
            "memcpy"
        } else if callee.starts_with("llvm.memmove.") {
            "memmove"
        } else if callee.starts_with("llvm.memset.") {
            "memset"
        } else if matches!(callee, "bcmp" | "memcmp") {
            callee
        } else {
            return Ok(None);
        };
        let args: Vec<_> = args.iter().filter(|a| !a.is_empty()).collect();
        if args.len() < 3 {
            return Err(format!("{callee} operands missing"));
        }
        let first = self.typed_operand(state, args[0])?;
        let aligns = [attribute_align(args[0]), attribute_align(args[1])];
        let length = self.typed_operand(state, args[2])?;
        if length.width()? != 64 {
            return Err("byte operation length must be 64-bit".into());
        }
        if kind == "memset" {
            let fill = self.typed_operand(state, args[1])?;
            self.fill(state, &first, aligns[0], &fill, &length)?;
            return Ok(Some(None));
        }
        let second = self.typed_operand(state, args[1])?;
        if matches!(kind, "memcpy" | "memmove") {
            self.copy(
                state,
                (&first, aligns[0]),
                (&second, aligns[1]),
                &length,
                kind == "memmove",
            )?;
            return Ok(Some(None));
        }
        Ok(Some(Some(self.compare_bytes(
            state,
            (&first, aligns[0]),
            (&second, aligns[1]),
            &length,
            kind,
        )?)))
    }

    /// Validity of touching `length` bytes at `pointer` whose call-site
    /// `align` attribute promises `alignment`. Zero length skips access
    /// checks only; calls.rs checks argument attributes before dispatch.
    fn range(
        &mut self,
        state: &State,
        (pointer, alignment): (&Value, u64),
        length: &Value,
    ) -> Result<String, String> {
        if literal(&length.expr) == Some(0) {
            return Ok("true".into());
        }
        let zero = format!("(= {} {})", length.expr, bv(0, 64));
        let pointer = self.resolve_pointer(state, pointer)?;
        let (name, offset) = pointer.pointer()?;
        let object = state.memory.get(name).ok_or("unknown allocation")?;
        let size = object.size.term();
        let end = format!("(bvadd {offset} {})", length.expr);
        let aligned = if alignment > 1 {
            format!(
                "(= (bvurem {} {}) {})",
                pointer.expr,
                bv(u128::from(alignment), 64),
                bv(0, 64)
            )
        } else {
            "true".into()
        };
        Ok(format!(
            "(or {zero} {})",
            and(&[
                pointer.defined.clone(),
                object.alive.to_string(),
                format!("(bvuge {end} {offset})"),
                format!("(bvule {end} {size})"),
                aligned,
            ])
        ))
    }

    fn copy(
        &mut self,
        state: &mut State,
        (target, target_align): (&Value, u64),
        (source, source_align): (&Value, u64),
        length: &Value,
        overlap: bool,
    ) -> Result<(), String> {
        let mut valid = vec![
            length.defined.clone(),
            self.range(state, (target, target_align), length)?,
            self.range(state, (source, source_align), length)?,
        ];
        let count = self.concrete(state, &length.expr)?;
        if count == Some(0) {
            return self.safety(state, &and(&valid), "invalid memory copy");
        }
        let target = self.resolve_pointer(state, target)?;
        let source = self.resolve_pointer(state, source)?;
        let (to, to_offset) = target.pointer()?;
        let (from, from_offset) = source.pointer()?;
        if state.memory.get(from).is_some_and(|o| o.writeonly) {
            self.safety(
                state,
                &format!("(= {} {})", length.expr, bv(0, 64)),
                "read through writeonly byte memory",
            )?;
            return Ok(());
        }
        if state
            .memory
            .get(from)
            .is_some_and(|o| o.stack && o.frame_live && !o.alive)
        {
            self.safety(state,&format!("(= {} {})",length.expr,bv(0,64)),"byte-intrinsic read of a stack object outside its lifetime: LangRef's load exception covers load instructions only")?;
        }
        if to == from && !overlap {
            // llvm.memcpy: ranges must be identical or disjoint.
            valid.push(format!(
                "(or (= {to_offset} {from_offset}) (bvule (bvadd {to_offset} {len}) {from_offset}) (bvule (bvadd {from_offset} {len}) {to_offset}))",
                len = length.expr
            ));
        }
        self.safety(state, &and(&valid), "invalid memory copy")?;
        if state.memory.get(to).is_some_and(|o| o.readonly) {
            self.safety(
                state,
                &format!("(= {} {})", length.expr, bv(0, 64)),
                "write through readonly byte memory",
            )?;
            return Ok(());
        }
        let snapshot = state.memory.get(from).ok_or("unknown allocation")?.clone();
        let destination = state.memory.get(to).ok_or("unknown allocation")?.clone();
        let (limit, guarded) = match count {
            Some(n) if n <= u128::from(COPY_LIMIT) => (n as u64, false),
            Some(_) => return Err("memory copy exceeds the modeled size".into()),
            None if destination.size.upper().min(snapshot.size.upper()) <= SYMBOLIC_COPY_LIMIT => {
                (destination.size.upper().min(snapshot.size.upper()), true)
            }
            None => {
                let bound = self
                    .length_bound(state, &length.expr, SYMBOLIC_COPY_LIMIT)?
                    .ok_or("symbolic copy length exceeds 256-byte finite model bound")?;
                (bound, true)
            }
        };
        // Writes go to a copy; reads see the contents before this copy.
        let mut written = destination.clone();
        let start = crate::heap::literal(to_offset).and_then(|n| u64::try_from(n).ok());
        // A whole-object copy at zero offsets is exactly the source arrays
        // on every accessible destination byte. Array tails are inaccessible;
        // realloc copies only the old extent, so no tail becomes observable.
        // Keep all range/permission/overlap obligations and shadow copying.
        let whole = !guarded
            && count == destination.size.concrete().map(u128::from)
            && crate::heap::literal(to_offset) == Some(0)
            && crate::heap::literal(from_offset) == Some(0);
        if whole {
            written.bytes = snapshot.bytes.clone();
            written.initialized = snapshot.initialized.clone();
            written.poison = snapshot.poison.clone();
            written.known = None;
        }
        for i in 0..if whole { 0 } else { limit } {
            let at = |offset: &str| format!("(bvadd {offset} {})", bv(u128::from(i), 64));
            let mut byte = format!("(select {} {})", snapshot.bytes, at(from_offset));
            let mut init = snapshot.init_at(&at(from_offset));
            let mut bad = snapshot.poison_at(&at(from_offset));
            if guarded {
                let inside = format!("(bvult {} {})", bv(u128::from(i), 64), length.expr);
                byte = format!(
                    "(ite {inside} {byte} (select {} {}))",
                    destination.bytes,
                    at(to_offset)
                );
                init = format!(
                    "(ite {inside} {init} {})",
                    destination.init_at(&at(to_offset))
                );
                bad = format!(
                    "(ite {inside} {bad} {})",
                    destination.poison_at(&at(to_offset))
                );
            }
            written.bytes = format!("(store {} {} {byte})", written.bytes, at(to_offset));
            written.set_flags(&at(to_offset), start.map(|t| t + i), &init, &bad);
        }
        // Whole pointer slots move even at symbolic offsets. Their guards
        // require the complete eight bytes to lie inside the source range;
        // the destination load still checks the address bytes exactly.
        let offsets = (
            self.concrete(state, to_offset)?,
            self.concrete(state, from_offset)?,
        );
        let object = state.memory.get_mut(to).ok_or("unknown allocation")?;
        object.bytes = written.bytes;
        object.initialized = written.initialized;
        object.poison = written.poison;
        object.known = written.known;
        match (offsets.0, count) {
            (Some(t), Some(n)) => object.forget(Some(t as u64), n as u64),
            _ => object.forget(None, 0),
        }
        object.forget_pointers(to_offset, &length.expr);
        for slot in &snapshot.pointers {
            let (offset, inside) = match (offsets, count, crate::heap::literal(&slot.offset)) {
                ((Some(t), Some(f)), Some(n), Some(p)) => {
                    if p < f || p + 8 > f + n {
                        continue;
                    }
                    (bv(t + (p - f), 64), "true".into())
                }
                _ => {
                    let relative = format!("(bvsub {} {from_offset})", slot.offset);
                    (
                        format!("(bvadd {to_offset} {relative})"),
                        and(&[
                            format!("(bvuge {} {})", length.expr, bv(8, 64)),
                            format!("(bvuge {} {from_offset})", slot.offset),
                            format!("(bvule {relative} (bvsub {} {}))", length.expr, bv(8, 64)),
                        ]),
                    )
                }
            };
            object.pointers.push(crate::memory::PointerSlot {
                offset,
                value: slot.value.clone(),
                valid: and(&[slot.valid.clone(), inside]),
            });
        }
        match (offsets, count) {
            ((Some(t), Some(f)), Some(n)) => {
                let (t, f, n) = (t as u64, f as u64, n as u64);
                // Whole stored values keep forwarding at their new offset.
                for (o, value) in &snapshot.values {
                    let w = value.width().map_or(u64::MAX, |w| u64::from(w / 8));
                    if *o >= f && o.saturating_add(w) <= f + n {
                        object.values.insert(t + (o - f), value.clone());
                    }
                }
                // Random-derived bytes stay marked where they land.
                object.tainted.retain(|o| *o < t || *o >= t + n);
                for o in snapshot.tainted.range(f..f + n) {
                    object.tainted.insert(t + (o - f));
                }
                object.tainted_anywhere |= snapshot.tainted_anywhere;
            }
            _ => {
                if snapshot.tainted_anywhere || !snapshot.tainted.is_empty() {
                    object.tainted_anywhere = true;
                }
            }
        }
        Ok(())
    }

    fn fill(
        &mut self,
        state: &mut State,
        target: &Value,
        alignment: u64,
        fill: &Value,
        length: &Value,
    ) -> Result<(), String> {
        let valid = and(&[
            length.defined.clone(),
            fill.defined.clone(),
            self.range(state, (target, alignment), length)?,
        ]);
        self.safety(state, &valid, "invalid memory fill")?;
        let count = self.concrete(state, &length.expr)?;
        if count == Some(0) {
            return Ok(());
        }
        let target = self.resolve_pointer(state, target)?;
        let (to, to_offset) = target.pointer()?;
        let destination = state.memory.get(to).ok_or("unknown allocation")?.clone();
        if destination.readonly {
            self.safety(
                state,
                &format!("(= {} {})", length.expr, bv(0, 64)),
                "write through readonly byte memory",
            )?;
            return Ok(());
        }
        let (limit, guarded) = match count {
            Some(n) if n <= u128::from(COPY_LIMIT) => (n as u64, false),
            Some(_) => return Err("memory fill exceeds the modeled size".into()),
            None if destination.size.upper() <= SYMBOLIC_COPY_LIMIT => {
                (destination.size.upper(), true)
            }
            None => {
                let bound = self
                    .length_bound(state, &length.expr, SYMBOLIC_COPY_LIMIT)?
                    .ok_or("symbolic fill length exceeds 256-byte finite model bound")?;
                (bound, true)
            }
        };
        let mut written = destination.clone();
        let first = crate::heap::literal(to_offset).and_then(|n| u64::try_from(n).ok());
        for i in 0..limit {
            let at = format!("(bvadd {to_offset} {})", bv(u128::from(i), 64));
            // The fill value is required defined, so filled bytes are not poison.
            let (byte, init, bad) = if guarded {
                let inside = format!("(bvult {} {})", bv(u128::from(i), 64), length.expr);
                (
                    format!(
                        "(ite {inside} {} (select {} {at}))",
                        fill.expr, destination.bytes
                    ),
                    format!("(ite {inside} true {})", destination.init_at(&at)),
                    format!("(ite {inside} false {})", destination.poison_at(&at)),
                )
            } else {
                (fill.expr.clone(), "true".into(), "false".into())
            };
            written.bytes = format!("(store {} {at} {byte})", written.bytes);
            written.set_flags(&at, first.map(|t| t + i), &init, &bad);
        }
        let start = self.concrete(state, to_offset)?;
        let object = state.memory.get_mut(to).ok_or("unknown allocation")?;
        object.bytes = written.bytes;
        object.initialized = written.initialized;
        object.poison = written.poison;
        object.known = written.known;
        match (start, count) {
            (Some(t), Some(n)) => object.forget(Some(t as u64), n as u64),
            _ => object.forget(None, 0),
        }
        object.forget_pointers(to_offset, &length.expr);
        Ok(())
    }

    /// bcmp: zero iff equal, otherwise an unspecified nonzero value.
    /// memcmp: only the sign of the first differing byte pair is specified.
    fn compare_bytes(
        &mut self,
        state: &mut State,
        (left, left_align): (&Value, u64),
        (right, right_align): (&Value, u64),
        length: &Value,
        kind: &str,
    ) -> Result<Value, String> {
        let valid = and(&[
            length.defined.clone(),
            self.range(state, (left, left_align), length)?,
            self.range(state, (right, right_align), length)?,
        ]);
        self.safety(state, &valid, "invalid memory comparison")?;
        let count = self.concrete(state, &length.expr)?;
        let (n, guarded) = match count {
            Some(n) if n <= u128::from(COPY_LIMIT) => (n as u64, false),
            Some(_) => return Err("memory comparison exceeds the 4096-byte model bound".into()),
            None => (
                self.length_bound(state, &length.expr, SYMBOLIC_COPY_LIMIT)?
                    .ok_or("symbolic comparison length exceeds 256-byte finite model bound")?,
                true,
            ),
        };
        let result = self.symbol("(_ BitVec 32)")?;
        let zero = bv(0, 32);
        if n == 0 {
            state.constraints.push(format!("(= {result} {zero})"));
            return Ok(Value::bits(result, 32, "true".into()));
        }
        let left = self.resolve_pointer(state, left)?;
        let right = self.resolve_pointer(state, right)?;
        let byte = |state: &State, pointer: &Value, i: u64| -> Result<(String, String), String> {
            let (name, offset) = pointer.pointer()?;
            let object = state.memory.get(name).ok_or("unknown allocation")?;
            let at = format!("(bvadd {offset} {})", bv(u128::from(i), 64));
            Ok((
                format!("(select {} {at})", object.bytes),
                and(&[
                    object.init_at(&at),
                    crate::value::not(&object.poison_at(&at)),
                ]),
            ))
        };
        for pointer in [&left, &right] {
            let (name, _) = pointer.pointer()?;
            if state.memory.get(name).is_some_and(|o| o.writeonly) {
                self.safety(
                    state,
                    &format!("(= {} {})", length.expr, bv(0, 64)),
                    "read through writeonly byte memory",
                )?;
            }
        }
        let mut equal = Vec::new();
        let mut initialized = Vec::new();
        // Walk backwards so each byte's outcome wraps the later ones.
        let mut order = zero.clone();
        for i in (0..n).rev() {
            let (a, ai) = byte(state, &left, i)?;
            let (b, bi) = byte(state, &right, i)?;
            let active = if guarded {
                format!("(bvult {} {})", bv(u128::from(i), 64), length.expr)
            } else {
                "true".into()
            };
            initialized.push(format!("(or (not {active}) {})", and(&[ai, bi])));
            equal.push(format!("(or (not {active}) (= {a} {b}))"));
            // Combine the guard and inequality so the prior order occurs
            // once. Nesting two ITEs duplicated it exponentially for n bytes.
            order = format!(
                "(ite (and {active} (not (= {a} {b}))) (ite (bvult {a} {b}) {} {}) {order})",
                bv(u128::from(u32::MAX), 32),
                bv(1, 32)
            );
        }
        let all = and(&equal);
        if kind == "bcmp" {
            state
                .constraints
                .push(format!("(= (= {result} {zero}) {all})"));
        } else {
            // Same sign as the lexicographic order (-1, 0, 1), any magnitude.
            state.constraints.push(format!(
                "(and (= (= {result} {zero}) (= {order} {zero})) (= (bvslt {result} {zero}) (= {order} {})))",
                bv(u128::from(u32::MAX), 32)
            ));
        }
        Ok(Value::bits(result, 32, and(&initialized)))
    }
}

/// The `align N` attribute of a call operand, 1 when absent.
fn attribute_align(operand: &str) -> u64 {
    let words: Vec<_> = operand.split_whitespace().collect();
    words
        .windows(2)
        .find(|w| w[0] == "align")
        .and_then(|w| w[1].parse().ok())
        .unwrap_or(1)
}
