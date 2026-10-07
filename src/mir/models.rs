//! Owned function resolution and narrowly identified primitive std models.
//! Missing bodies, ambiguous methods, destructor effects and general FFI
//! stay unknown instead of being replaced with assumed behavior.

use super::{
    Data, Node, State, Verifier, integer,
    parse::{Function, base_type, split},
};
use std::collections::BTreeMap;

type Resolved = Option<(Function, BTreeMap<String, String>)>;

fn callee(text: &str) -> (String, Vec<String>) {
    if let Some((base, types)) = text.rsplit_once("::<").filter(|_| text.ends_with('>')) {
        (
            base.into(),
            split(types.trim_end_matches('>'))
                .into_iter()
                .map(str::to_owned)
                .collect(),
        )
    } else {
        (text.into(), Vec::new())
    }
}

impl Verifier {
    pub fn resolve(&self, state: &State, name: &str, args: &[Node]) -> Result<Resolved, String> {
        let (name, generics) = callee(name);
        let name = state.ty(&name);
        let namespace = state
            .frame()?
            .function
            .rsplit_once("::")
            .map(|(n, _)| n)
            .unwrap_or("");
        let qualified = if name.contains("::") || namespace.is_empty() {
            name.clone()
        } else {
            format!("{namespace}::{name}")
        };
        let mut candidates = Vec::new();
        if let Some(f) = self.program.functions.get(&qualified) {
            candidates.push(f)
        } else if let Some(f) = self.program.functions.get(&name) {
            candidates.push(f)
        } else if let Some((owner, leaf)) = name.rsplit_once("::") {
            // A method must identify its owner in a receiver or return type.
            // Same-name methods with no identifying type remain unsupported.
            for f in self
                .program
                .functions
                .values()
                .filter(|f| f.name.rsplit("::").next() == Some(leaf))
            {
                if f.params.iter().any(|(_, ty)| {
                    base_type(ty.trim_start_matches("&mut ").trim_start_matches('&')) == owner
                }) || base_type(&f.result) == owner
                {
                    candidates.push(f)
                }
            }
        }
        if candidates.len() > 1 {
            return Err(format!("unmodeled call {name}: ambiguous owned function"));
        }
        let Some(function) = candidates.first() else {
            return Ok(None);
        };
        if function.params.len() != args.len() {
            return Err("MIR call signature count mismatch".into());
        }
        let mut bindings = BTreeMap::new();
        if !generics.is_empty() {
            let names = self
                .program
                .registry
                .generics
                .get(&function.key)
                .or_else(|| self.program.registry.generics.get(&function.name))
                .ok_or("unsupported MIR generic declaration")?;
            if names.len() != generics.len() {
                return Err("unsupported MIR generic arity".into());
            }
            for (name, ty) in names.iter().zip(generics) {
                bindings.insert(name.clone(), state.ty(&ty));
            }
        }
        Ok(Some(((*function).clone(), bindings)))
    }
    pub fn model(
        &mut self,
        state: &mut State,
        name: &str,
        mut args: Vec<Node>,
    ) -> Result<Node, String> {
        if let Some(result) = self.collection(state, name, &args)? {
            return Ok(result);
        }
        if name.starts_with("std::intrinsics::transmute::<")
            || name.starts_with("core::intrinsics::transmute::<")
        {
            let (_, types) = callee(name);
            if args.len() != 1 || types.len() != 2 {
                return Err("unsupported MIR transmute signature".into());
            }
            self.compatible(&types[0], &args[0].ty)?;
            let target = &types[1];
            let value = args[0].value()?;
            if target == "bool" && args[0].ty == "u8" {
                self.obligation(
                    state,
                    &format!(
                        "(or (= {} {}) (= {} {}))",
                        value.expr,
                        crate::value::bv(0, 8),
                        value.expr,
                        crate::value::bv(1, 8)
                    ),
                    "MIR invalid bit pattern for bool",
                )?;
                return Ok(Node::scalar(
                    "bool",
                    crate::value::Value::bits(
                        format!("((_ extract 0 0) {})", value.expr),
                        1,
                        value.defined.clone(),
                    ),
                ));
            }
            if target == "char" && args[0].ty == "u32" {
                self.obligation(
                    state,
                    &format!(
                        "(and (bvule {} {}) (or (bvult {} {}) (bvugt {} {})))",
                        value.expr,
                        crate::value::bv(0x10ffff, 32),
                        value.expr,
                        crate::value::bv(0xd800, 32),
                        value.expr,
                        crate::value::bv(0xdfff, 32)
                    ),
                    "MIR invalid bit pattern for char",
                )?;
                return Ok(Node::scalar(target, value.clone()));
            }
            if let (Some((a, _)), Some((b, _))) = (integer(&args[0].ty), integer(target))
                && a == b
                && !matches!(target.as_str(), "bool" | "char")
            {
                return Ok(Node::scalar(target, value.clone()));
            }
            return Err("unsupported MIR transmute".into());
        }
        if let Some(inner) = name
            .strip_prefix('<')
            .and_then(|s| s.strip_suffix(" as From<u8>>::from"))
        {
            if self.program.registry.traits.contains("From") {
                return Err(format!(
                    "unmodeled call {name}: owned From trait needs a body"
                ));
            }
            if args.len() != 1 || args[0].ty != "u8" {
                return Err("unsupported MIR From signature".into());
            }
            return self.cast(args.remove(0), inner);
        }
        if let Some((ty, method)) = name.rsplit_once("::") {
            if integer(ty).is_some()
                && args.len() == 2
                && matches!(method, "wrapping_add" | "wrapping_sub" | "wrapping_mul")
            {
                let a = &args[0];
                let b = &args[1];
                self.compatible(ty, &a.ty)?;
                self.compatible(ty, &b.ty)?;
                let operation = method
                    .strip_prefix("wrapping_")
                    .ok_or("MIR wrapping operation missing")?;
                return Ok(Node::scalar(
                    ty,
                    crate::value::binary(operation, &[], a.value()?, b.value()?)?,
                ));
            }
            if matches!(
                method,
                "unwrap" | "unwrap_or" | "is_some" | "is_none" | "is_ok" | "is_err"
            ) {
                let receiver = args.first().cloned().ok_or("MIR enum receiver missing")?;
                let receiver = if let Data::Ref(address) = &receiver.kind {
                    self.read_address(state, address)?
                } else {
                    receiver
                };
                let base = base_type(&receiver.ty);
                let standard = base.starts_with("std::option::")
                    || base.starts_with("core::option::")
                    || base.starts_with("std::result::")
                    || base.starts_with("core::result::");
                if standard
                    && base_type(ty).rsplit("::").next() == base.rsplit("::").next()
                    && !ty.contains(" as ")
                {
                    let Data::Enum {
                        variant,
                        mut fields,
                        ..
                    } = receiver.kind
                    else {
                        return Err("MIR enum std model requires enum".into());
                    };
                    if matches!(method, "is_some" | "is_none" | "is_ok" | "is_err") {
                        let yes = match method {
                            "is_some" => variant == "Some",
                            "is_none" => variant == "None",
                            "is_ok" => variant == "Ok",
                            _ => variant == "Err",
                        };
                        return Ok(Node::scalar(
                            "bool",
                            crate::value::constant(if yes { "true" } else { "false" }, 1)?,
                        ));
                    }
                    if matches!(variant.as_str(), "Some" | "Ok") {
                        return fields.pop().ok_or("MIR enum payload absent".into());
                    }
                    if method == "unwrap_or" && args.len() == 2 {
                        return Ok(args.remove(1));
                    }
                    self.obligation(
                        state,
                        "false",
                        &format!("Rust panic call {name} on {variant}"),
                    )?;
                }
            }
        }
        Err(format!("unmodeled call {name}"))
    }
}
