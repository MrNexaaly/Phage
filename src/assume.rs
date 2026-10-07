//! llvm.assume operand bundles. Guarantees are checked as UB obligations,
//! never added as trusted path constraints. Unlisted bundle tags stay unknown.
use crate::{
    engine::{Engine, State},
    ir::{split, typed},
    symbols::matching_paren,
    value::{and, bv, not},
};

impl Engine {
    pub fn assume_bundles(&mut self, state: &mut State, text: &str) -> Result<(), String> {
        let text = text
            .trim()
            .strip_suffix(']')
            .ok_or("unsupported unterminated assume operand bundles")?;
        for bundle in split(text) {
            let open = bundle
                .find('(')
                .ok_or("unsupported assume operand bundle syntax")?;
            let close =
                matching_paren(bundle, open).ok_or("unsupported assume operand bundle syntax")?;
            if !bundle[close + 1..].trim().is_empty() {
                return Err("unsupported assume operand bundle suffix".into());
            }
            let tag = bundle[..open].trim();
            if tag == "\"ignore\"" {
                continue;
            }
            let args = split(&bundle[open + 1..close]);
            if tag != "\"nonnull\"" || args.len() != 1 || typed(args[0])?.0 != "ptr" {
                return Err(format!("unsupported llvm.assume operand bundle {bundle}"));
            }
            let pointer = self.typed_operand(state, args[0])?;
            // nonnull on poison/undef cannot justify a successful property.
            self.safety(
                state,
                &and(&[
                    pointer.defined.clone(),
                    not(&format!("(= {} {})", pointer.expr, bv(0, 64))),
                ]),
                "llvm.assume nonnull guarantee violated (undefined behavior)",
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
