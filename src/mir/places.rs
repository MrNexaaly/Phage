//! Typed local storage and references with frame/generation identities.
//! Moves invalidate only the selected place. Symbolic array accesses are
//! bounds checked; expired references and uninitialized reads fail explicitly.

use super::{
    Address, Data, Node, Projection, State, Verifier,
    parse::{closing, parentheses},
};
use crate::value::{Value, and, bv};

#[derive(Debug)]
enum Part {
    Deref,
    Field(usize),
    Index(String),
    Variant(String),
}
fn place(text: &str) -> Result<(String, Vec<Part>), String> {
    let mut text = parentheses(text.trim());
    // Type annotations occur outside the nested place expression.
    let mut depth = 0;
    for (i, c) in text.char_indices() {
        if matches!(c, '(' | '[') {
            depth += 1
        } else if matches!(c, ')' | ']') {
            depth -= 1
        }
        if c == ':' && depth == 0 && text[i..].starts_with(": ") {
            text = text[..i].trim();
            break;
        }
    }
    text = parentheses(text);
    let mut nesting = 0;
    let mut downcast = None;
    for (i, c) in text.char_indices() {
        if matches!(c, '(' | '[') {
            nesting += 1;
        } else if matches!(c, ')' | ']') {
            nesting -= 1;
        }
        if nesting == 0 && text[i..].starts_with(" as ") {
            downcast = Some(i);
        }
    }
    if let Some(i) = downcast {
        let (base, variant) = (&text[..i], &text[i + 4..]);
        let (mut local, mut parts) = place(base)?;
        parts.push(Part::Variant(variant.into()));
        return Ok((std::mem::take(&mut local), parts));
    }
    let (local, mut parts, mut rest) = if text.starts_with('(') {
        let end = closing(text, 0).ok_or("MIR place parentheses unclosed")?;
        let (local, parts) = place(&text[1..end])?;
        (local, parts, &text[end + 1..])
    } else if let Some(inner) = text.strip_prefix('*') {
        let (local, mut parts) = place(inner)?;
        parts.push(Part::Deref);
        return Ok((local, parts));
    } else {
        let end = text.find(['.', '[']).unwrap_or(text.len());
        let local = &text[..end];
        if !local.starts_with('_') || !local[1..].chars().all(|c| c.is_ascii_digit()) {
            return Err(format!("unsupported MIR place {text}"));
        }
        (local.into(), Vec::new(), &text[end..])
    };
    while !rest.is_empty() {
        if let Some(field) = rest.strip_prefix('.') {
            let end = field.find(['.', '[']).unwrap_or(field.len());
            parts.push(Part::Field(
                field[..end]
                    .parse()
                    .map_err(|_| "MIR field index invalid")?,
            ));
            rest = &field[end..];
        } else if rest.starts_with('[') {
            let end = closing(rest, 0).ok_or("MIR index unclosed")?;
            parts.push(Part::Index(rest[1..end].into()));
            rest = &rest[end + 1..];
        } else {
            return Err(format!("unsupported MIR projection {rest}"));
        }
    }
    Ok((local, parts))
}

impl Verifier {
    pub fn address_valid(&mut self, state: &State, address: &Address) -> Result<(), String> {
        let local = state
            .locals
            .get(&(address.frame, address.local.clone()))
            .ok_or("MIR referenced local missing")?;
        if !local.alive || local.generation != address.generation {
            self.obligation(state, "false", "MIR expired reference or local access")?;
        }
        Ok(())
    }
    pub fn address(&mut self, state: &mut State, text: &str) -> Result<Address, String> {
        let (local, parts) = place(text)?;
        let frame = state.frame()?.id;
        let slot = state
            .locals
            .get(&(frame, local.clone()))
            .ok_or_else(|| format!("MIR local {local} missing"))?;
        let mut address = Address {
            frame,
            local,
            generation: slot.generation,
            projections: Vec::new(),
        };
        self.address_valid(state, &address)?;
        for part in parts {
            match part {
                Part::Deref => {
                    let value = self.read_address(state, &address)?;
                    let Data::Ref(pointer) = value.kind else {
                        return Err("unsupported MIR dereference of non-reference".into());
                    };
                    address = pointer;
                    self.address_valid(state, &address)?;
                }
                Part::Field(i) => address.projections.push(Projection::Field(i)),
                Part::Variant(v) => address.projections.push(Projection::Variant(v)),
                Part::Index(local) => {
                    let node = self.operand(state, &format!("copy {local}"))?;
                    let index = node.value()?.clone();
                    address.projections.push(Projection::Index(index));
                }
            }
        }
        Ok(address)
    }
    pub fn read_place(
        &mut self,
        state: &mut State,
        text: &str,
        moving: bool,
    ) -> Result<Node, String> {
        let address = self.address(state, text)?;
        let value = self.read_address(state, &address)?;
        if moving {
            self.write_address(state, &address, Node::uninit(&value.ty))?;
        }
        Ok(value)
    }
    pub fn read_address(&mut self, state: &State, address: &Address) -> Result<Node, String> {
        self.address_valid(state, address)?;
        let node = state
            .locals
            .get(&(address.frame, address.local.clone()))
            .ok_or("MIR local missing")?
            .value
            .clone();
        let node = self.project(state, node, &address.projections)?;
        self.observe(state, &node)?;
        Ok(node)
    }
    fn project(
        &mut self,
        state: &State,
        mut node: Node,
        projections: &[Projection],
    ) -> Result<Node, String> {
        for projection in projections {
            node = match projection {
                Projection::Variant(expected) => {
                    if let Data::Enum { variant, .. } = &node.kind {
                        if variant != expected {
                            self.obligation(state, "false", "MIR wrong enum variant accessed")?;
                        }
                    } else {
                        return Err("MIR downcast requires an enum".into());
                    }
                    node
                }
                Projection::Field(index) => {
                    let fields = match &node.kind {
                        Data::Tuple(v) | Data::Struct(v) => v,
                        Data::Enum { fields, .. } => fields,
                        Data::Uninit => {
                            self.obligation(state, "false", "MIR uninitialized aggregate access")?;
                            return Err("infeasible MIR uninitialized access".into());
                        }
                        _ => return Err("unsupported MIR field projection".into()),
                    };
                    fields
                        .get(*index)
                        .cloned()
                        .ok_or("MIR field out of range")?
                }
                Projection::Index(index) => {
                    let values = match &node.kind {
                        Data::Array(v) | Data::Vector { elements: v, .. } => v,
                        _ => {
                            return Err("unsupported MIR indexing of non-array".into());
                        }
                    };
                    self.obligation(
                        state,
                        &and(&[
                            index.defined.clone(),
                            format!(
                                "(bvult {} {})",
                                index.expr,
                                bv(values.len() as u128, index.width()?)
                            ),
                        ]),
                        "MIR array index out of bounds",
                    )?;
                    let mut selected = values.last().cloned().ok_or("MIR empty array access")?;
                    for (i, value) in values.iter().enumerate().rev().skip(1) {
                        selected = self.select(
                            &format!("(= {} {})", index.expr, bv(i as u128, index.width()?)),
                            value,
                            &selected,
                        )?;
                    }
                    selected
                }
            };
        }
        Ok(node)
    }
    pub fn write_address(
        &mut self,
        state: &mut State,
        address: &Address,
        node: Node,
    ) -> Result<(), String> {
        self.address_valid(state, address)?;
        let key = (address.frame, address.local.clone());
        let mut root = state
            .locals
            .get(&key)
            .ok_or("MIR local missing")?
            .value
            .clone();
        self.update(state, &mut root, &address.projections, node)?;
        state.locals.get_mut(&key).ok_or("MIR local missing")?.value = root;
        Ok(())
    }
    fn update(
        &mut self,
        state: &State,
        root: &mut Node,
        path: &[Projection],
        node: Node,
    ) -> Result<(), String> {
        let Some((projection, rest)) = path.split_first() else {
            *root = node;
            return Ok(());
        };
        match projection {
            Projection::Variant(expected) => {
                if let Data::Enum { variant, .. } = &root.kind {
                    if variant != expected {
                        self.obligation(state, "false", "MIR wrong enum variant written")?;
                    }
                } else {
                    return Err("MIR enum write requires active variant".into());
                }
                self.update(state, root, rest, node)
            }
            Projection::Field(index) => {
                let fields = match &mut root.kind {
                    Data::Tuple(v) | Data::Struct(v) => v,
                    Data::Enum { fields, .. } => fields,
                    _ => return Err("unsupported MIR field write".into()),
                };
                self.update(
                    state,
                    fields
                        .get_mut(*index)
                        .ok_or("MIR field write out of range")?,
                    rest,
                    node,
                )
            }
            Projection::Index(index) => {
                let values = match &mut root.kind {
                    Data::Array(v) | Data::Vector { elements: v, .. } => v,
                    _ => {
                        return Err("unsupported MIR array write".into());
                    }
                };
                self.obligation(
                    state,
                    &format!(
                        "(bvult {} {})",
                        index.expr,
                        bv(values.len() as u128, index.width()?)
                    ),
                    "MIR array write out of bounds",
                )?;
                for (i, old) in values.iter_mut().enumerate() {
                    let mut new = old.clone();
                    self.update(state, &mut new, rest, node.clone())?;
                    *old = self.select(
                        &format!("(= {} {})", index.expr, bv(i as u128, index.width()?)),
                        &new,
                        old,
                    )?;
                }
                Ok(())
            }
        }
    }
    pub fn select(&self, condition: &str, yes: &Node, no: &Node) -> Result<Node, String> {
        if yes.ty != no.ty {
            return Err("unsupported MIR selection across types".into());
        }
        let kind = match (&yes.kind, &no.kind) {
            (Data::Scalar(a), Data::Scalar(b)) => Data::Scalar(Value::bits(
                format!("(ite {condition} {} {})", a.expr, b.expr),
                a.width()?,
                format!("(ite {condition} {} {})", a.defined, b.defined),
            )),
            (Data::Tuple(a), Data::Tuple(b))
            | (Data::Array(a), Data::Array(b))
            | (Data::Struct(a), Data::Struct(b))
                if a.len() == b.len() =>
            {
                let fields = a
                    .iter()
                    .zip(b)
                    .map(|(a, b)| self.select(condition, a, b))
                    .collect::<Result<Vec<_>, _>>()?;
                match &yes.kind {
                    Data::Array(_) => Data::Array(fields),
                    Data::Struct(_) => Data::Struct(fields),
                    _ => Data::Tuple(fields),
                }
            }
            _ => return Err("unsupported MIR symbolic selection of references or variants".into()),
        };
        Ok(Node {
            ty: yes.ty.clone(),
            kind,
        })
    }
}
