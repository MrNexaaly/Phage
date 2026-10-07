//! Source-level executor over pinned rustc built MIR. Each path owns its
//! frames and typed local storage. Unsupported Rust behavior is unknown.

mod collections;
mod eval;
pub mod frontend;
mod models;
pub mod parse;
mod places;
pub mod registry;
mod run;
mod steps;

use crate::{engine::Engine, value::Value};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct Node {
    pub ty: String,
    pub kind: Data,
}
#[derive(Clone, Debug)]
pub enum Data {
    Uninit,
    Scalar(Value),
    Tuple(Vec<Node>),
    Array(Vec<Node>),
    Struct(Vec<Node>),
    Enum {
        variant: String,
        tag: i128,
        fields: Vec<Node>,
    },
    Ref(Address),
    Vector {
        elements: Vec<Node>,
        capacity: Value,
    },
    String(Vec<Node>),
    Str {
        bytes: Vec<Node>,
        owner: Option<Address>,
    },
}
impl Node {
    pub fn scalar(ty: &str, value: Value) -> Self {
        Self {
            ty: ty.into(),
            kind: Data::Scalar(value),
        }
    }
    pub fn uninit(ty: &str) -> Self {
        Self {
            ty: ty.into(),
            kind: Data::Uninit,
        }
    }
    pub fn value(&self) -> Result<&Value, String> {
        match &self.kind {
            Data::Scalar(v) => Ok(v),
            Data::Uninit => Err("MIR uninitialized value read".into()),
            _ => Err(format!("unsupported scalar operation on {}", self.ty)),
        }
    }
}
#[derive(Clone, Debug)]
pub enum Projection {
    Field(usize),
    Index(Value),
    Variant(String),
}
#[derive(Clone, Debug)]
pub struct Address {
    pub frame: usize,
    pub local: String,
    pub generation: usize,
    pub projections: Vec<Projection>,
}
#[derive(Clone)]
pub struct Local {
    pub value: Node,
    pub alive: bool,
    pub generation: usize,
}
#[derive(Clone)]
pub struct Frame {
    pub id: usize,
    pub function: String,
    pub block: String,
    pub bindings: BTreeMap<String, String>,
    pub resume: Option<(Address, String)>,
    pub visits: BTreeMap<String, usize>,
}
#[derive(Clone)]
pub struct State {
    pub frames: Vec<Frame>,
    pub locals: BTreeMap<(usize, String), Local>,
    pub constraints: Vec<String>,
}
impl State {
    pub fn frame(&self) -> Result<&Frame, String> {
        self.frames.last().ok_or("MIR frame missing".into())
    }
    pub fn frame_mut(&mut self) -> Result<&mut Frame, String> {
        self.frames.last_mut().ok_or("MIR frame missing".into())
    }
    pub fn ty(&self, ty: &str) -> String {
        let mut result = ty.to_owned();
        if let Ok(frame) = self.frame() {
            for (name, value) in &frame.bindings {
                result = replace_word(&result, name, value);
            }
        }
        result
    }
}
fn replace_word(text: &str, name: &str, value: &str) -> String {
    let mut result = String::new();
    let mut word = String::new();
    for c in text.chars().chain(std::iter::once(' ')) {
        if c.is_alphanumeric() || c == '_' {
            word.push(c);
        } else {
            if word == name {
                result.push_str(value);
            } else {
                result.push_str(&word);
            }
            word.clear();
            result.push(c);
        }
    }
    result.pop();
    result
}

pub struct Verifier {
    pub core: Engine,
    pub program: parse::Program,
    pub next_frame: usize,
    pub depth: usize,
    pub assumptions: std::collections::BTreeSet<String>,
}
impl Verifier {
    pub fn new(
        program: parse::Program,
        timeout: u64,
        visits: usize,
        states: usize,
        depth: usize,
    ) -> Result<Self, String> {
        Ok(Self {
            core: Engine::new(timeout, visits, states)?,
            program,
            next_frame: 0,
            depth,
            assumptions: Default::default(),
        })
    }
    pub fn obligation(&mut self, state: &State, valid: &str, detail: &str) -> Result<(), String> {
        use crate::{solver::Sat, value::not};
        match self.core.solver.check(&state.constraints, &not(valid))? {
            Sat::No => Ok(()),
            Sat::Unknown => Err(format!("solver unknown: {}", self.core.solver.last_unknown)),
            Sat::Yes => Err(format!(
                "COUNTEREXAMPLE:{detail}\n{}",
                self.core
                    .solver
                    .model(&state.constraints, &not(valid), &self.core.inputs)?
            )),
        }
    }
    pub fn observe(&mut self, state: &State, node: &Node) -> Result<(), String> {
        match &node.kind {
            Data::Uninit => self.obligation(state, "false", "MIR uninitialized value read"),
            Data::Scalar(v) => self.obligation(state, &v.defined, "MIR invalid scalar read"),
            Data::Tuple(v) | Data::Array(v) | Data::Struct(v) => {
                for item in v {
                    self.observe(state, item)?;
                }
                Ok(())
            }
            Data::Enum { fields, .. } => {
                for item in fields {
                    self.observe(state, item)?;
                }
                Ok(())
            }
            Data::Ref(a) => {
                self.address_valid(state, a)?;
                Ok(())
            }
            Data::Vector { elements, .. } | Data::String(elements) => {
                for item in elements {
                    self.observe(state, item)?;
                }
                Ok(())
            }
            Data::Str { bytes, owner } => {
                if let Some(address) = owner {
                    self.read_address(state, address)?;
                }
                for byte in bytes {
                    self.observe(state, byte)?;
                }
                Ok(())
            }
        }
    }
}

pub fn integer(ty: &str) -> Option<(u32, bool)> {
    if ty == "bool" {
        return Some((1, false));
    }
    if ty == "char" {
        return Some((32, false));
    }
    if ty == "usize" {
        return Some((64, false));
    }
    if ty == "isize" {
        return Some((64, true));
    }
    let signed = ty.starts_with('i');
    if !signed && !ty.starts_with('u') {
        return None;
    }
    ty[1..]
        .parse::<u32>()
        .ok()
        .filter(|w| matches!(w, 8 | 16 | 32 | 64 | 128))
        .map(|w| (w, signed))
}
