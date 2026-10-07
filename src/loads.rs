//! Plain/atomic scalar, pointer and vector loads. Dead stack loads yield
//! poison after address/bounds/alignment checks; !noundef observes it as UB.
use crate::{
    engine::{Engine, State},
    ir::split,
    memory,
    value::{Kind, Value},
};
impl Engine {
    pub fn load_instruction(
        &mut self,
        state: &mut State,
        rest: &str,
        noundef: bool,
    ) -> Result<(Value, bool), String> {
        let args = split(rest);
        let ty = args.first().ok_or("load type missing")?;
        let pointer = self.typed_operand(state, args.get(1).ok_or("load pointer missing")?)?;
        let pointer = self.dereference(state, &pointer)?;
        let alignment = crate::execute::align(&args, ty)?;
        if crate::vectors::vector_type(ty).is_some() {
            return Ok((
                self.vector_load(state, ty, &pointer, alignment, noundef)?,
                false,
            ));
        }
        let float = crate::floats::format(ty);
        let width = if *ty == "ptr" {
            64
        } else if let Some((e, s)) = float {
            e + s
        } else {
            crate::execute::integer_width(ty)?
        };
        let valid = memory::access_load(
            &state.memory,
            &pointer,
            u64::from(width.div_ceil(8)),
            alignment,
        )?;
        self.safety(state, &valid, "invalid memory read")?;
        let pointer = Value {
            defined: "true".into(),
            ..pointer
        };
        let (object, _) = pointer.pointer()?;
        let dead = state
            .memory
            .get(object)
            .is_some_and(|o| o.stack && !o.alive);
        let loaded = if *ty == "ptr" {
            if dead {
                Value {
                    expr: crate::value::bv(0, 64),
                    kind: Kind::Pointer {
                        object: String::new(),
                        offset: crate::value::bv(0, 64),
                        bounded: false,
                    },
                    defined: "false".into(),
                }
            } else {
                self.load_pointer(state, &pointer)?
            }
        } else {
            memory::load(&state.memory, &pointer, width)?
        };
        if noundef {
            self.safety(
                state,
                &loaded.defined,
                "uninitialized or poison memory read",
            )?;
        }
        let copied = if !dead && *ty != "ptr" && width == 64 && float.is_none() {
            self.copied_pointer(state, &pointer, &loaded)?
        } else {
            None
        };
        let value = match (copied, float) {
            (Some(copied), _) => Value {
                kind: Kind::PointerBits(Box::new(copied)),
                ..loaded
            },
            (None, Some(format)) => self.bits_float(&loaded, format)?,
            _ => loaded,
        };
        Ok((
            value,
            !dead && self.loads_taint(state, &pointer, u64::from(width.div_ceil(8))),
        ))
    }
}
