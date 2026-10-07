//! Allocator ABI, exact symbolic lengths, Layout obligations and realloc.
use super::*;
use crate::memory::Length;
const COPY_LIMIT: u64 = 4096;
const SYMBOLIC_LIMIT: u64 = 1 << 20;
impl Engine {
    fn concrete_u64(&mut self, state: &State, value: &Value, what: &str) -> Result<u64, String> {
        self.concrete(state, &value.expr)?
            .and_then(|n| u64::try_from(n).ok())
            .ok_or_else(|| format!("symbolic {what} is unsupported"))
    }

    /// A finite exact upper bound under the current path, used only for
    /// exploration. Solver uncertainty never adds a guessed bound.
    pub fn length_bound(
        &mut self,
        state: &State,
        term: &str,
        cap: u64,
    ) -> Result<Option<u64>, String> {
        if let Some(n) = literal(term) {
            return Ok(u64::try_from(n).ok().filter(|n| *n <= cap));
        }
        match self.solver.check(
            &state.constraints,
            &format!("(bvugt {term} {})", bv(u128::from(cap), 64)),
        )? {
            Sat::Yes => return Ok(None),
            Sat::Unknown => return Err("solver unknown checking symbolic length bound".into()),
            Sat::No => {}
        }
        let mut upper = cap;
        while upper > 1 {
            let half = upper / 2;
            match self.solver.check(
                &state.constraints,
                &format!("(bvugt {term} {})", bv(u128::from(half), 64)),
            )? {
                Sat::No => upper = half,
                Sat::Yes => break,
                Sat::Unknown => return Err("solver unknown narrowing symbolic length bound".into()),
            }
        }
        Ok(Some(upper))
    }
    fn allocation_length(&mut self, state: &State, term: &str) -> Result<Length, String> {
        if let Some(n) = self
            .concrete(state, term)?
            .and_then(|n| u64::try_from(n).ok())
        {
            return Ok(n.into());
        }
        let upper = self
            .length_bound(state, term, SYMBOLIC_LIMIT)?
            .ok_or("symbolic heap length exceeds the 1048576-byte placement/model bound")?;
        Ok(Length::Symbolic {
            term: term.into(),
            upper,
        })
    }

    /// Handles allocator ABI calls; `None` when `callee` is not one.
    pub fn allocator(
        &mut self,
        state: &mut State,
        callee: &str,
        args: &[&str],
    ) -> Result<Option<Option<Value>>, String> {
        let Some(kind) = allocator_kind(callee) else {
            return Ok(None);
        };
        let mut values = Vec::new();
        for arg in args.iter().filter(|a| !a.is_empty()) {
            values.push(self.typed_operand(state, arg)?);
        }
        let result = match (kind, values.as_slice()) {
            ("__rust_no_alloc_shim_is_unstable_v2", []) => None,
            ("__rust_alloc" | "__rust_alloc_zeroed", [size, align]) => {
                Some(self.heap_object(state, size, align, kind == "__rust_alloc_zeroed", None)?)
            }
            ("__rust_dealloc", [pointer, size, align]) => {
                self.release(state, pointer, size, align)?;
                None
            }
            ("__rust_realloc", [pointer, size, align, new_size]) => {
                let old = self.release(state, pointer, size, align)?;
                Some(self.heap_object(state, new_size, align, false, Some(old))?)
            }
            _ => return Err(format!("unexpected allocator call {callee}")),
        };
        Ok(Some(result))
    }

    fn heap_object(
        &mut self,
        state: &mut State,
        size: &Value,
        align: &Value,
        zeroed: bool,
        previous: Option<Object>,
    ) -> Result<Value, String> {
        // GlobalAlloc: a zero-size request is undefined behavior.
        self.safety(
            state,
            &and(&[
                size.defined.clone(),
                align.defined.clone(),
                format!("(not (= {} {}))", size.expr, bv(0, 64)),
            ]),
            "zero-size or poison heap allocation",
        )?;
        if size.width()? != 64 || align.width()? != 64 {
            return Err("unsupported allocator size/alignment width".into());
        }
        let alignment = self.concrete_u64(state, align, "allocation alignment")?;
        if alignment == 0 || !alignment.is_power_of_two() {
            self.safety(state, "false", "invalid heap allocation Layout alignment")?;
        }
        let largest = (i64::MAX as u64).saturating_sub(alignment.saturating_sub(1));
        self.safety(
            state,
            &format!("(bvule {} {})", size.expr, bv(u128::from(largest), 64)),
            "heap allocation exceeds Layout/isize::MAX size limit",
        )?;
        let size = self.allocation_length(state, &size.expr)?;
        self.assumptions
            .insert("heap allocation succeeds (the allocator never returns null)".into());
        let align = alignment;
        let name = format!("heap{}", self.fresh);
        let bytes = if zeroed {
            format!(
                "((as const (Array (_ BitVec 64) (_ BitVec 8))) {})",
                bv(0, 8)
            )
        } else {
            self.symbol("(Array (_ BitVec 64) (_ BitVec 8))")?
        };
        let pointer = self.allocation_extent(
            state,
            name.clone(),
            (size.clone(), align),
            bytes,
            false,
            true,
        )?;
        {
            let object = state.memory.get_mut(&name).ok_or("heap object missing")?;
            object.heap = true;
            if zeroed {
                object.initialized = crate::memory::ALL_DEFINED.into();
                object.known = None;
            }
        }
        if let Some(old) = previous {
            // Only [0, min(old, new)) survives; later bytes are fresh and
            // uninitialized even if the old object had more (shrink, zeroed).
            let keep = old.size.min(&size);
            if keep.upper() > COPY_LIMIT {
                return Err("reallocation preserves more than the modeled size".into());
            }
            let object = state.memory.get_mut(&name).ok_or("heap object missing")?;
            let fresh = object.clone();
            for i in 0..keep.upper() {
                let at = bv(u128::from(i), 64);
                let inside = format!("(bvult {at} {})", keep.term());
                let byte = format!(
                    "(ite {inside} (select {} {at}) (select {} {at}))",
                    old.bytes, fresh.bytes
                );
                let initialized =
                    format!("(ite {inside} {} {})", old.init_at(&at), fresh.init_at(&at));
                let poison = format!(
                    "(ite {inside} {} {})",
                    old.poison_at(&at),
                    fresh.poison_at(&at)
                );
                object.bytes = format!("(store {} {at} {byte})", object.bytes);
                object.set_flags(&at, Some(i), &initialized, &poison);
            }
            object.pointers = old
                .pointers
                .into_iter()
                .filter_map(|mut slot| {
                    if keep.upper() < 8 {
                        return None;
                    }
                    slot.valid = and(&[
                        slot.valid,
                        format!("(bvuge {} {})", keep.term(), bv(8, 64)),
                        format!(
                            "(bvule {} (bvsub {} {}))",
                            slot.offset,
                            keep.term(),
                            bv(8, 64)
                        ),
                    ]);
                    Some(slot)
                })
                .collect();
            object.values = old
                .values
                .into_iter()
                .filter(|(offset, value)| {
                    let width = value.width().map_or(u64::MAX, |w| u64::from(w / 8));
                    keep.concrete()
                        .is_some_and(|n| offset.saturating_add(width) <= n)
                })
                .collect();
            object.tainted = old.tainted.range(..keep.upper()).copied().collect();
            object.tainted_anywhere = old.tainted_anywhere;
        }
        Ok(pointer)
    }

    /// Checks a deallocation against the allocation's layout and frees it.
    fn release(
        &mut self,
        state: &mut State,
        pointer: &Value,
        size: &Value,
        align: &Value,
    ) -> Result<Object, String> {
        let pointer = self.resolve_pointer(state, pointer)?;
        let (name, offset) = pointer.pointer()?;
        let (name, offset) = (name.to_owned(), offset.to_owned());
        let object = state.memory.get(&name).ok_or("unknown allocation")?.clone();
        let valid = and(&[
            pointer.defined.clone(),
            size.defined.clone(),
            align.defined.clone(),
            object.heap.to_string(),
            object.alive.to_string(),
            format!("(= {offset} {})", bv(0, 64)),
            format!("(= {} {})", size.expr, object.size.term()),
            format!(
                "(= {} {})",
                align.expr,
                bv(u128::from(object.alignment), 64)
            ),
        ]);
        self.safety(
            state,
            &valid,
            "invalid deallocation (layout, double free or non-heap)",
        )?;
        state
            .memory
            .get_mut(&name)
            .ok_or("unknown allocation")?
            .alive = false;
        Ok(object)
    }
}
