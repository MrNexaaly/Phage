//! Bounded safe Vec and String API models. Capacity is an API guarantee,
//! not a guessed allocator result. Allocation success is an explicit report
//! assumption; custom allocators, raw buffers and unmodeled APIs are unknown.

use super::{Address, Data, Node, Projection, State, Verifier, integer, parse::base_type};
use crate::{
    solver::Sat,
    value::{Value, bv},
};

fn unit() -> Node {
    Node {
        ty: "()".into(),
        kind: Data::Tuple(Vec::new()),
    }
}
fn length(value: usize) -> Node {
    Node::scalar(
        "usize",
        Value::bits(bv(value as u128, 64), 64, "true".into()),
    )
}
fn boolean(value: bool) -> Node {
    Node::scalar(
        "bool",
        Value::bits(bv(u128::from(value), 1), 1, "true".into()),
    )
}
fn option(ty: &str, value: Option<Node>) -> Node {
    let (variant, tag, fields) = if let Some(v) = value {
        ("Some", 1, vec![v])
    } else {
        ("None", 0, Vec::new())
    };
    Node {
        ty: format!("std::option::Option<{ty}>"),
        kind: Data::Enum {
            variant: variant.into(),
            tag,
            fields,
        },
    }
}

impl Verifier {
    fn allocation(
        &mut self,
        state: &State,
        requested: &Value,
        element_size: u64,
    ) -> Result<(), String> {
        if self.program.registry.custom_allocator {
            return Err("unsupported MIR custom allocator".into());
        }
        self.obligation(
            state,
            &format!(
                "(bvule {} {})",
                requested.expr,
                bv((i64::MAX as u64 / element_size) as u128, 64)
            ),
            "Rust capacity overflow",
        )?;
        if self.core.solver.check(
            &state.constraints,
            &format!("(bvugt {} {})", requested.expr, bv(4096, 64)),
        )? != Sat::No
        {
            return Err("reachable MIR allocation bound of 4096 elements exceeded".into());
        }
        self.assumptions.insert(
            "bounded standard allocations succeed; allocator failure is not analyzed".into(),
        );
        Ok(())
    }
    fn capacity(&mut self, state: &mut State, minimum: &Value, size: u64) -> Result<Value, String> {
        let symbol = self.core.symbol("(_ BitVec 64)")?;
        state
            .constraints
            .push(format!("(bvuge {symbol} {})", minimum.expr));
        state.constraints.push(format!(
            "(bvule {symbol} {})",
            bv((i64::MAX as u64 / size) as u128, 64)
        ));
        Ok(Value::bits(symbol, 64, "true".into()))
    }
    fn owned(&mut self, state: &State, node: &Node) -> Result<(Node, Option<Address>), String> {
        let mut node = node.clone();
        let mut address = None;
        for _ in 0..32 {
            if let Data::Ref(reference) = node.kind {
                node = self.read_address(state, &reference)?;
                address = Some(reference)
            } else {
                return Ok((node, address));
            }
        }
        Err("unsupported MIR reference nesting".into())
    }
    fn bytes(&mut self, state: &State, node: &Node) -> Result<Vec<Node>, String> {
        let (node, _) = self.owned(state, node)?;
        match node.kind {
            Data::Str { bytes, .. } | Data::String(bytes) => Ok(bytes),
            _ => Err("unsupported MIR string argument".into()),
        }
    }
    pub fn collection(
        &mut self,
        state: &mut State,
        name: &str,
        args: &[Node],
    ) -> Result<Option<Node>, String> {
        let Some((owner, method)) = name.rsplit_once("::") else {
            return Ok(None);
        };
        let base = base_type(owner);
        let leaf = base.rsplit("::").next().unwrap_or(&base);
        let known_namespace = !base.contains("::")
            || base.starts_with("std::")
            || base.starts_with("core::")
            || base.starts_with("alloc::");
        let shadowed = self
            .program
            .registry
            .definitions
            .keys()
            .any(|k| k.rsplit("::").next() == Some(leaf));
        if known_namespace
            && !shadowed
            && leaf == "Vec"
            && matches!(method, "new" | "with_capacity")
        {
            let element = owner
                .split_once("::<")
                .and_then(|(_, s)| s.strip_suffix('>'))
                .ok_or("MIR Vec type missing")?;
            let (width, _) = integer(element).ok_or("unsupported MIR Vec element type")?;
            let size = u64::from(width.div_ceil(8));
            let capacity = if method == "new" && args.is_empty() {
                Value::bits(bv(0, 64), 64, "true".into())
            } else if method == "with_capacity" && args.len() == 1 {
                self.compatible("usize", &args[0].ty)?;
                let requested = args[0].value()?;
                self.allocation(state, requested, size)?;
                self.capacity(state, requested, size)?
            } else {
                return Err("unsupported MIR Vec constructor signature".into());
            };
            return Ok(Some(Node {
                ty: format!("std::vec::Vec<{element}>"),
                kind: Data::Vector {
                    elements: Vec::new(),
                    capacity,
                },
            }));
        }
        if known_namespace && !shadowed && leaf == "String" && method == "new" && args.is_empty() {
            return Ok(Some(Node {
                ty: "std::string::String".into(),
                kind: Data::String(Vec::new()),
            }));
        }
        if name == "<String as From<&str>>::from"
            || name == "<std::string::String as From<&str>>::from"
        {
            if self.program.registry.traits.contains("From") {
                return Err("unmodeled call with ambiguous owned From trait".into());
            }
            if args.len() != 1 {
                return Err("MIR String From signature mismatch".into());
            }
            let bytes = self.bytes(state, &args[0])?;
            self.allocation(state, length(bytes.len()).value()?, 1)?;
            return Ok(Some(Node {
                ty: "std::string::String".into(),
                kind: Data::String(bytes),
            }));
        }
        if (name.starts_with("<String as PartialEq<")
            || name.starts_with("<std::string::String as PartialEq<"))
            && matches!(method, "eq" | "ne")
        {
            if self.program.registry.traits.contains("PartialEq") {
                return Err("unmodeled call with ambiguous owned PartialEq trait".into());
            }
            if args.len() != 2 {
                return Err("MIR string equality signature unsupported".into());
            }
            let a = self.bytes(state, &args[0])?;
            let b = self.bytes(state, &args[1])?;
            let equal = if a.len() != b.len() {
                "false".into()
            } else {
                crate::value::and(
                    &a.iter()
                        .zip(&b)
                        .map(|(a, b)| Ok(format!("(= {} {})", a.value()?.expr, b.value()?.expr)))
                        .collect::<Result<Vec<_>, String>>()?,
                )
            };
            let equal = if method == "eq" {
                equal
            } else {
                crate::value::not(&equal)
            };
            return Ok(Some(Node::scalar(
                "bool",
                Value::bits(
                    format!("(ite {equal} {} {})", bv(1, 1), bv(0, 1)),
                    1,
                    "true".into(),
                ),
            )));
        }
        let Some(receiver) = args.first() else {
            return Ok(None);
        };
        let (node, address) = self.owned(state, receiver)?;
        if let Data::Vector {
            mut elements,
            capacity,
        } = node.kind.clone()
        {
            let element = node
                .ty
                .split_once('<')
                .and_then(|(_, s)| s.strip_suffix('>'))
                .ok_or("MIR Vec element type absent")?;
            let size = u64::from(
                integer(element)
                    .ok_or("unsupported MIR Vec element")?
                    .0
                    .div_ceil(8),
            );
            if name.starts_with("<Vec<") && name.contains(" as Index<usize>>::index") {
                if self.program.registry.traits.contains("Index") {
                    return Err("unmodeled call with ambiguous owned Index trait".into());
                }
                let index = args.get(1).ok_or("MIR Vec index absent")?.value()?;
                self.obligation(
                    state,
                    &format!("(bvult {} {})", index.expr, bv(elements.len() as u128, 64)),
                    "Rust Vec index out of bounds",
                )?;
                let mut address = address.ok_or("MIR Vec index requires reference")?;
                address.projections.push(Projection::Index(index.clone()));
                return Ok(Some(Node {
                    ty: format!("&{element}"),
                    kind: Data::Ref(address),
                }));
            }
            if !known_namespace || shadowed || leaf != "Vec" {
                return Ok(None);
            }
            match method {
                "len" => return Ok(Some(length(elements.len()))),
                "is_empty" => return Ok(Some(boolean(elements.is_empty()))),
                "capacity" => return Ok(Some(Node::scalar("usize", capacity.clone()))),
                "push" => {
                    let value = args.get(1).ok_or("MIR Vec push value absent")?;
                    self.compatible(element, &value.ty)?;
                    if elements.len() >= 4096 {
                        return Err("reachable MIR Vec element bound exceeded".into());
                    }
                    self.allocation(state, length(elements.len() + 1).value()?, size)?;
                    let minimum = length(elements.len() + 1);
                    let growth = self.capacity(state, minimum.value()?, size)?;
                    let capacity = Value::bits(
                        format!(
                            "(ite (bvuge {} {}) {} {})",
                            capacity.expr,
                            minimum.value()?.expr,
                            capacity.expr,
                            growth.expr
                        ),
                        64,
                        "true".into(),
                    );
                    elements.push(value.clone());
                    self.write_address(
                        state,
                        &address.ok_or("MIR Vec mutation requires reference")?,
                        Node {
                            ty: node.ty.clone(),
                            kind: Data::Vector { elements, capacity },
                        },
                    )?;
                    return Ok(Some(unit()));
                }
                "pop" => {
                    let value = elements.pop();
                    self.write_address(
                        state,
                        &address.ok_or("MIR Vec mutation requires reference")?,
                        Node {
                            ty: node.ty.clone(),
                            kind: Data::Vector {
                                elements,
                                capacity: capacity.clone(),
                            },
                        },
                    )?;
                    return Ok(Some(option(element, value)));
                }
                _ => {}
            }
        }
        if let Data::String(mut bytes) = node.kind {
            if !known_namespace || shadowed || leaf != "String" {
                return Ok(None);
            }
            match method {
                "len" => return Ok(Some(length(bytes.len()))),
                "is_empty" => return Ok(Some(boolean(bytes.is_empty()))),
                "push_str" => {
                    let extra =
                        self.bytes(state, args.get(1).ok_or("MIR push_str argument absent")?)?;
                    if bytes.len() + extra.len() > 4096 {
                        return Err("reachable MIR String byte bound exceeded".into());
                    }
                    self.allocation(state, length(bytes.len() + extra.len()).value()?, 1)?;
                    bytes.extend(extra);
                    self.write_address(
                        state,
                        &address.ok_or("MIR String mutation requires reference")?,
                        Node {
                            ty: node.ty,
                            kind: Data::String(bytes),
                        },
                    )?;
                    return Ok(Some(unit()));
                }
                "as_str" => {
                    return Ok(Some(Node {
                        ty: "&str".into(),
                        kind: Data::Str {
                            bytes,
                            owner: address,
                        },
                    }));
                }
                _ => {}
            }
        }
        Ok(None)
    }
}
