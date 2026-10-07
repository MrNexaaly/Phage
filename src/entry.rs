//! Entry-domain construction from reference-shaped LLVM parameters. Bytes
//! are one unconstrained SMT array, materialized only by loads or witnesses.
use crate::{
    engine::{Engine, State},
    ir::Function,
    memory::{ALL_DEFINED, Length},
    value::{Value, bv},
};

#[derive(Clone)]
pub struct Buffer {
    pub label: String,
    pub bytes: String,
    pub capacity: u64,
}

fn attribute_number(attributes: &str, prefix: &str) -> Result<Option<u64>, String> {
    attributes
        .split_whitespace()
        .find_map(|word| {
            word.strip_prefix(prefix).map(|tail| {
                tail.strip_suffix(')')
                    .ok_or("malformed entry attribute")?
                    .parse()
                    .map_err(|_| "invalid entry attribute".to_owned())
            })
        })
        .transpose()
}

impl Engine {
    pub fn entry_arguments(
        &mut self,
        function: &Function,
        state: &mut State,
    ) -> Result<(), String> {
        for (i, (name, width)) in function.params.iter().enumerate() {
            if *width == 0 || *width > 128 {
                return Err("entry width is outside 1..128".into());
            }
            let input = format!("input{i}");
            let sort = format!("(_ BitVec {width})");
            self.solver.declare(&input, &sort)?;
            self.simplifier.declared(&input, &sort);
            self.inputs.push(input.clone());
            self.input_labels.push(name.trim_start_matches('%').into());
            state
                .env
                .insert(name.clone(), Value::bits(input, *width, "true".into()));
        }
        for (i, (name, ty)) in function.arguments.iter().enumerate() {
            if state.env.contains_key(name) {
                continue;
            }
            let attributes = &function.parameter_attributes[i];
            let words: Vec<_> = attributes.split_whitespace().collect();
            if ty != "ptr" || !words.contains(&"noalias") || !words.contains(&"noundef") {
                return Err(format!(
                    "unsupported entry argument: {ty} {name}; requires a noalias noundef bounded reference"
                ));
            }
            // Address spaces, byval/sret and pointer contracts with different
            // storage meanings are never silently treated as byte references.
            if words.iter().any(|w| {
                w.starts_with("byval")
                    || w.starts_with("sret")
                    || w.starts_with("inalloca")
                    || w.starts_with("preallocated")
                    || w.starts_with("dereferenceable_or_null")
            }) {
                return Err(format!(
                    "unsupported entry pointer attributes: {attributes}"
                ));
            }
            let alignment = match words.iter().position(|w| *w == "align") {
                Some(p) => words
                    .get(p + 1)
                    .ok_or("missing entry alignment")?
                    .parse()
                    .map_err(|_| "invalid entry alignment")?,
                None => 1,
            };
            let extent = attribute_number(attributes, "dereferenceable(")?;
            let size = if let Some(n) = extent {
                Length::Concrete(n)
            } else {
                let length_name = name.strip_suffix(".0").map(|stem| format!("{stem}.1"));
                let length_name = length_name
                    .filter(|n| function.arguments.get(i + 1) == Some(&(n.clone(), "i64".into())))
                    .or_else(|| unnamed_slice_length(function, i));
                let Some(length_name) = length_name.filter(|_| words.contains(&"nonnull")) else {
                    return Err(format!(
                        "unsupported entry argument: ptr {name}; no dereferenceable extent or recognized slice pair"
                    ));
                };
                let upper = self
                    .max_slice_len
                    .ok_or("slice entry requires explicit --maxSliceLen K")?;
                let length = &state
                    .env
                    .get(&length_name)
                    .ok_or("missing slice length")?
                    .expr;
                state
                    .constraints
                    .push(format!("(bvule {length} {})", bv(u128::from(upper), 64)));
                Length::Symbolic {
                    term: length.clone(),
                    upper,
                }
            };
            if size.upper() > 1_048_576 {
                return Err("entry byte buffer exceeds 1 MiB model bound".into());
            }
            let bytes = format!("input_bytes{i}");
            self.solver
                .declare(&bytes, "(Array (_ BitVec 64) (_ BitVec 8))")?;
            let pointer = self.allocation_extent(
                state,
                format!("entry:{name}"),
                (size.clone(), alignment),
                bytes.clone(),
                false,
                true,
            )?;
            let object = state
                .memory
                .get_mut(&format!("entry:{name}"))
                .ok_or("entry allocation missing")?;
            object.readonly = words.contains(&"readonly");
            object.unique = true;
            object.writeonly = words.contains(&"writeonly");
            object.initialized = ALL_DEFINED.into();
            object.known = None;
            state.env.insert(name.clone(), pointer);
            self.buffers.push(Buffer {
                label: name.trim_start_matches('%').into(),
                bytes,
                capacity: size.upper(),
            });
            self.assumptions.insert(format!("entry {name}: fresh nonaliasing initialized byte object, exact extent {}, alignment {alignment}, readonly {}", size.term(), words.contains(&"readonly")));
        }
        Ok(())
    }

    pub fn buffer_witness(&mut self, names: &mut Vec<String>) -> Result<String, String> {
        let mut mapping = String::new();
        for buffer in &self.buffers {
            mapping.push_str(&format!(
                "buffer {}: {} concrete bytes (capacity; slice length is its scalar input)\n",
                buffer.label, buffer.capacity
            ));
            for offset in 0..buffer.capacity {
                let name = format!("witness_{}_{}", buffer.bytes, offset);
                self.solver.define(
                    &name,
                    "(_ BitVec 8)",
                    &format!("(select {} {})", buffer.bytes, bv(u128::from(offset), 64)),
                )?;
                mapping.push_str(&format!("{name}={}[{offset}]\n", buffer.label));
                names.push(name);
            }
        }
        Ok(mapping)
    }
}

#[cfg(test)]
mod tests;

/// rustc sometimes drops argument names (`%0`, `%1`). A slice reference is then still recognizable: an unnamed
/// pointer followed by an unnamed `i64` that carries rustc's slice-length range `[0, isize::MAX]`.
fn unnamed_slice_length(function: &Function, i: usize) -> Option<String> {
    let unnamed = |n: &str| {
        n.strip_prefix('%')
            .is_some_and(|d| !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit()))
    };
    let (pointer, _) = function.arguments.get(i)?;
    let (length, ty) = function.arguments.get(i + 1)?;
    let attributes = function.parameter_attributes.get(i + 1)?;
    (unnamed(pointer)
        && unnamed(length)
        && ty == "i64"
        && attributes.contains("noundef")
        && attributes.contains("range(i64 0, -9223372036854775808)"))
    .then(|| length.clone())
}
