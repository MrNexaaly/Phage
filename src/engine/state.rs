//! Exploration state, suspended frames, verdicts and engine-owned caches.
use super::*;

#[derive(Clone)]
pub struct State {
    /// Current function and the module whose private symbols it sees.
    pub function: String,
    pub module: String,
    pub block: String,
    pub predecessor: String,
    /// Next instruction of `block`; nonzero only when resuming after a call.
    pub index: usize,
    pub env: BTreeMap<String, Value>,
    pub memory: Memory,
    pub constraints: Vec<String>,
    /// Block visits in the current call; each call counts its own loops,
    /// as in the MIR engine, so repeated calls do not share one bound.
    pub visits: BTreeMap<String, usize>,
    pub frames: Vec<Frame>,
    /// Set when a state is requeued without a feasibility query: after a
    /// call, a return or a literal branch.
    pub feasible: bool,
    /// Exact definitions of the opaque (uninterpreted) values on this path,
    /// used only to confirm counterexamples (see opaque.rs).
    pub abstractions: Vec<String>,
    /// Path constraints covered by the last satisfiable feasibility query.
    /// Inherited feasibility may skip later ones, so a completed path is
    /// counted only after they are checked too: never a vacuous proof.
    pub checked: usize,
    /// Stack objects allocated by the current frame; they die at its return.
    pub locals: Vec<String>,
}

/// A suspended caller: its SSA environment and where to resume.
#[derive(Clone)]
pub struct Frame {
    pub function: String,
    pub module: String,
    pub block: String,
    pub predecessor: String,
    pub index: usize,
    pub env: BTreeMap<String, Value>,
    pub destination: Option<String>,
    pub locals: Vec<String>,
    /// The caller's block visits, restored at return.
    pub visits: BTreeMap<String, usize>,
    /// The call site's return attributes (`noundef`, `range(...)`, ...).
    pub result_attributes: String,
}
pub enum Step {
    Continue,
    Jump(String),
    Branch(Value, String, String),
    /// Scrutinee, (case constant, label) pairs and the default label.
    Switch(Value, Vec<(String, String)>, String),
    Return(Option<Value>),
    Panic(String),
    /// Callee, evaluated arguments, the caller's result register, and the
    /// call site's return attributes.
    Call(Rc<Function>, Vec<Value>, Option<String>, String),
}
#[derive(Debug)]
pub struct Verdict {
    pub status: String,
    pub detail: String,
    pub model: String,
    pub states: usize,
    pub queries: usize,
    pub completed: usize,
}
pub struct Engine {
    pub solver: Solver,
    pub fresh: usize,
    pub inputs: Vec<String>,
    pub max_slice_len: Option<u64>,
    pub buffers: Vec<crate::entry::Buffer>,
    pub input_labels: Vec<String>,
    pub unwind: usize,
    pub max_states: usize,
    /// Declared functions whose attributes include `noreturn`.
    pub noreturn: BTreeSet<String>,
    /// Modules searched for callee bodies: the checked crate first.
    pub modules: Vec<Rc<Module>>,
    pub functions: BTreeMap<String, Rc<Function>>,
    pub call_depth: usize,
    /// Standard library bodies, loaded on the first unresolved call.
    pub library: Library,
    /// Modeling assumptions that a verdict actually relied on.
    pub assumptions: BTreeSet<String>,
    /// Memory key of a global referenced from one module but defined in
    /// another, by the referencing `qualify(module, symbol)` key.
    pub global_keys: BTreeMap<String, String>,
    /// Symbols derived from the OS random source (see opaque.rs).
    pub tainted: BTreeSet<String>,
    /// Uninterpreted functions already declared to the solver.
    pub opaque_functions: BTreeSet<String>,
    /// Global `define-fun` names by sort and expression (operands.rs).
    pub definitions: std::collections::HashMap<String, String>,
    /// Canonical XOR forms by term and by defined name (canonical.rs).
    pub xor_forms: std::collections::HashMap<String, crate::canonical::XorForm>,
    /// getrandom symbols and widths, fixed when sampling witnesses (opaque.rs).
    pub random_sources: Vec<(String, u32)>,
    /// Known-bits simplification state (knownbits.rs): `define-fun` bodies,
    /// bit-vector symbol widths, known bits and parsed bodies by name.
    pub simplifier: crate::knownbits::State,
    /// Shapes of opaque terms (opaque.rs).
    pub shapes: crate::opaque::Shapes,
    pub lifetimes: BTreeMap<String, BTreeSet<String>>,
}

pub enum Library {
    Disabled,
    Pending(Box<crate::toolchain::Toolchain>),
    Open(crate::library::Library),
    Failed(String),
}

impl Engine {
    /// Retain exploration counts when RSS sampling stops verification.
    pub fn memory_limit(&self, states: usize, completed: usize) -> Option<Verdict> {
        self.solver.budget.check().err().map(|detail| Verdict {
            status: "unknown".into(),
            detail,
            model: String::new(),
            states,
            queries: self.solver.queries,
            completed,
        })
    }
}
