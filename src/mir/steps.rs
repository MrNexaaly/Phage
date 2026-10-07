//! Built-MIR statements and terminators. Storage generations, runtime edges
//! and assertions are explicit; fake borrow-checker edges never run.

use super::{
    Data, Node, State, Verifier,
    parse::{closing, split},
    run::Step,
};
use crate::ir::Line;

fn successor(text: &str, kind: &str) -> Result<String, String> {
    let tail = text
        .split_once(&format!("{kind}: "))
        .ok_or("MIR successor missing")?
        .1;
    Ok(tail
        .split([',', ']'])
        .next()
        .ok_or("MIR successor missing")?
        .trim()
        .into())
}

impl Verifier {
    pub fn statement(&mut self, state: &mut State, line: &Line) -> Result<Step, String> {
        let text = line.text.as_str();
        for (marker, live) in [("StorageLive(", true), ("StorageDead(", false)] {
            if let Some(local) = text.strip_prefix(marker).and_then(|s| s.strip_suffix(')')) {
                let key = (state.frame()?.id, local.into());
                let slot = state
                    .locals
                    .get_mut(&key)
                    .ok_or("MIR storage local missing")?;
                if live {
                    slot.generation += 1;
                    slot.value = Node::uninit(&slot.value.ty);
                }
                slot.alive = live;
                return Ok(Step::Next);
            }
        }
        if let Some(args) = text
            .strip_prefix("FakeRead(")
            .and_then(|s| s.strip_suffix(')'))
        {
            let args = split(args);
            let place = args.last().ok_or("MIR FakeRead place absent")?;
            self.read_place(state, place, false)?;
            return Ok(Step::Next);
        }
        if let Some(place) = text
            .strip_prefix("PlaceMention(")
            .and_then(|s| s.strip_suffix(')'))
        {
            self.address(state, place)?;
            return Ok(Step::Next);
        }
        if text.starts_with("AscribeUserType(") {
            return Ok(Step::Next);
        } // Type constraint already checked by rustc.
        if let Some(target) = text.strip_prefix("goto -> ") {
            return Ok(Step::Jump(target.into()));
        }
        if text.starts_with("falseEdge -> ") || text.starts_with("falseUnwind -> ") {
            return Ok(Step::Jump(successor(text, "real")?));
        }
        if text == "return" {
            return Ok(Step::Return);
        }
        if matches!(text, "unreachable" | "resume" | "abort") {
            return Ok(Step::Panic(format!("reachable MIR {text}")));
        }
        if let Some(tail) = text.strip_prefix("switchInt(") {
            let close = closing(text, "switchInt".len()).ok_or("MIR switch operand unclosed")?;
            let operand = &text["switchInt(".len()..close];
            let value = self.operand(state, operand)?;
            let branches = tail
                .split_once(" -> [")
                .ok_or("MIR switch targets missing")?
                .1
                .trim_end_matches(']');
            let mut choices = Vec::new();
            let mut otherwise = None;
            for branch in split(branches) {
                let (n, target) = branch.split_once(": ").ok_or("MIR switch branch invalid")?;
                if n == "otherwise" {
                    otherwise = Some(target.into())
                } else {
                    choices.push((n.into(), target.into()))
                }
            }
            return Ok(Step::Switch(
                value,
                choices,
                otherwise.ok_or("MIR switch otherwise missing")?,
            ));
        }
        if text.starts_with("assert(") {
            let close = closing(text, "assert".len()).ok_or("MIR assert operands unclosed")?;
            let args = split(&text["assert(".len()..close]);
            let cond = args.first().ok_or("MIR assertion condition missing")?;
            let (expected, operand) = cond.strip_prefix('!').map_or((true, *cond), |s| (false, s));
            let value = self.operand(state, operand)?;
            let message = args.get(1).unwrap_or(&"assertion").trim_matches('"').into();
            return Ok(Step::Assert(
                value,
                expected,
                message,
                successor(text, "success")?,
            ));
        }
        if text.starts_with("drop(") {
            let close = closing(text, 4).ok_or("MIR drop place unclosed")?;
            let address = self.address(state, &text[5..close])?;
            let slot = state
                .locals
                .get(&(address.frame, address.local.clone()))
                .ok_or("MIR drop local missing")?;
            if self.needs_drop(&slot.value) {
                return Err(format!("unsupported MIR destructor for {}", slot.value.ty));
            }
            self.write_address(state, &address, Node::uninit(&slot.value.ty.clone()))?;
            return Ok(Step::Jump(successor(text, "return")?));
        }
        if let Some((destination, rvalue)) = text.split_once(" = ") {
            if super::parse::arrow(rvalue).is_some() {
                let open = rvalue.find('(').ok_or("MIR callee arguments missing")?;
                let close = closing(rvalue, open).ok_or("MIR callee arguments unclosed")?;
                let name = rvalue[..open].trim();
                // Panics terminate this proof obligation regardless of the
                // formatting payload. No application function is stubbed.
                if name.starts_with("core::panicking::") || name.starts_with("std::panicking::") {
                    return Ok(Step::Panic(format!("Rust panic call {name}")));
                }
                let target = successor(rvalue, "return")?;
                let address = self.address(state, destination)?;
                let args = split(&rvalue[open + 1..close])
                    .into_iter()
                    .filter(|s| !s.is_empty())
                    .map(|s| self.operand(state, s))
                    .collect::<Result<Vec<_>, _>>()?;
                return Ok(Step::Call(name.into(), args, address, target));
            }
            let address = self.address(state, destination)?;
            let ty = self.place_type(state, &address)?;
            let value = self.expression(state, rvalue, &ty)?;
            self.compatible(&ty, &value.ty)?;
            let value = self.bind(state, value)?;
            self.write_address(state, &address, value)?;
            return Ok(Step::Next);
        }
        Err(format!("unsupported instruction in MIR: {text}"))
    }
    fn place_type(&mut self, state: &State, address: &super::Address) -> Result<String, String> {
        let root = state
            .locals
            .get(&(address.frame, address.local.clone()))
            .ok_or("MIR destination local missing")?;
        if address.projections.is_empty() {
            return Ok(root.value.ty.clone());
        }
        // Field writes have an existing initialized aggregate; selecting its
        // current value also checks the projection and its declared type.
        Ok(self.read_address(state, address)?.ty)
    }
    pub fn needs_drop(&self, node: &Node) -> bool {
        if matches!(
            node.kind,
            Data::Uninit | Data::Ref(_) | Data::Scalar(_) | Data::Str { .. }
        ) {
            return false;
        }
        if self.program.registry.has_drop(&node.ty) {
            return true;
        }
        match &node.kind {
            Data::Tuple(v) | Data::Array(v) | Data::Struct(v) => {
                v.iter().any(|v| self.needs_drop(v))
            }
            Data::Enum { fields, .. } => fields.iter().any(|v| self.needs_drop(v)),
            Data::Vector { elements, .. } | Data::String(elements) => {
                elements.iter().any(|v| self.needs_drop(v))
            }
            _ => false,
        }
    }
}
