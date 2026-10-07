//! Operating-system calls that std makes on the property's behalf, modeled by
//! their documented effect under a recorded environment assumption (as heap
//! allocation success is). Only calls without a definition in the program are
//! modeled; anything not listed stays an unmodeled call, which is unknown.

use crate::{
    engine::{Engine, State},
    memory,
    value::{Value, and, bv},
};

/// Largest buffer filled with fresh bytes in one call.
const FILL_LIMIT: u64 = 4096;

impl Engine {
    /// Handles a modeled OS call; `None` when `callee` is not one.
    pub fn environment_call(
        &mut self,
        state: &mut State,
        callee: &str,
        args: &[&str],
    ) -> Result<Option<Option<Value>>, String> {
        if callee != "getrandom" {
            return Ok(None);
        }
        // ssize_t getrandom(void *buf, size_t len, unsigned flags)
        let args: Vec<_> = args.iter().filter(|a| !a.is_empty()).collect();
        let [buffer, length, flags] = args.as_slice() else {
            return Err("getrandom takes three arguments".into());
        };
        let buffer = self.typed_operand(state, buffer)?;
        let length = self.typed_operand(state, length)?;
        let flags = self.typed_operand(state, flags)?;
        self.safety(
            state,
            &and(&[
                buffer.defined.clone(),
                length.defined.clone(),
                flags.defined.clone(),
            ]),
            "poison getrandom argument",
        )?;
        let size = self
            .concrete(state, &length.expr)?
            .and_then(|n| u64::try_from(n).ok())
            .filter(|n| *n <= FILL_LIMIT)
            .ok_or("getrandom of a symbolic or large length is unsupported")?;
        self.assumptions.insert(
            "the OS random source (getrandom) fills the whole buffer with arbitrary bytes".into(),
        );
        if size > 0 {
            let pointer = self.resolve_pointer(state, &buffer)?;
            let valid = memory::access_store(&state.memory, &pointer, size, 1)?;
            self.safety(state, &valid, "invalid getrandom buffer")?;
            let width = u32::try_from(size * 8).map_err(|e| e.to_string())?;
            let random = self.symbol(&format!("(_ BitVec {width})"))?;
            // Random bytes seed hash keys: values mixed from them are
            // opaque functions (opaque.rs).
            self.tainted.insert(random.clone());
            self.random_sources.push((random.clone(), width));
            memory::store(
                &mut state.memory,
                &pointer,
                &Value::bits(random, width, "true".into()),
            )?;
            self.mark_taint(state, &pointer, size, true)?;
        }
        Ok(Some(Some(Value::bits(
            bv(u128::from(size), 64),
            64,
            "true".into(),
        ))))
    }
}

#[cfg(test)]
mod tests;
