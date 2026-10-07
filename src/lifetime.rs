//! Whole stack lifetimes: classify marker targets before any allocation or
//! access, propagate same-module callee parameter effects to callers, and
//! refuse unresolved associations. Poison markers have no effect.
use crate::{
    engine::{Engine, State, Step},
    ir::{Function, Module, split, typed},
    solver::Sat,
    symbols::matching_paren,
    value::bv,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

#[derive(Clone, Debug, PartialEq, Eq)]
enum Root {
    Stack(String),
    Parameter(String),
    Poison,
    Other,
    Unknown,
}
#[derive(Default)]
struct Plan {
    roots: Vec<(Root, bool)>,
    calls: Vec<(String, Vec<Root>)>,
}

fn operand(text: &str) -> &str {
    text.split_whitespace().last().unwrap_or("")
}
fn root(
    name: &str,
    f: &Function,
    defs: &BTreeMap<String, String>,
    seen: &mut BTreeSet<String>,
) -> Root {
    if name == "poison" {
        return Root::Poison;
    }
    if name.starts_with('@') || name == "null" {
        return Root::Other;
    }
    if f.arguments.iter().any(|(n, t)| n == name && t == "ptr") {
        return Root::Parameter(name.into());
    }
    if !seen.insert(name.into()) {
        return Root::Unknown;
    }
    let Some(code) = defs.get(name) else {
        return Root::Unknown;
    };
    if code.starts_with("alloca ") {
        return Root::Stack(name.into());
    }
    if let Some(rest) = code.strip_prefix("getelementptr ") {
        let args = split(rest);
        if args.len() >= 3 && args[2..].iter().all(|a| operand(a) == "0") {
            return root(operand(args[1]), f, defs, seen);
        }
    }
    if let Some(rest) = code
        .strip_prefix("bitcast ptr ")
        .and_then(|r| r.strip_suffix(" to ptr"))
    {
        return root(operand(rest), f, defs, seen);
    }
    if let Some(rest) = code.strip_prefix("select ") {
        let args = split(rest);
        if args.len() == 3 {
            return match operand(args[0]) {
                "poison" => Root::Poison,
                "true" => root(operand(args[1]), f, defs, seen),
                "false" => root(operand(args[2]), f, defs, seen),
                _ => Root::Unknown,
            };
        }
    }
    if code.contains("@__rust_alloc(") || code.contains("@__rust_alloc_zeroed(") {
        return Root::Other;
    }
    Root::Unknown
}

fn scan(f: &Function, module: &Module) -> Result<Plan, String> {
    let defs: BTreeMap<_, _> = f
        .blocks
        .values()
        .flatten()
        .filter_map(|l| l.text.split_once(" = "))
        .map(|(n, c)| (n.to_owned(), c.to_owned()))
        .collect();
    let mut plan = Plan::default();
    for line in f.blocks.values().flatten() {
        let Some((_, call)) = line.text.split_once("call ") else {
            continue;
        };
        let Some((at, open)) = crate::calls::callee_span(call) else {
            continue;
        };
        // An SSA indirect callee %f is not the unrelated symbol @f.
        // Unclassified caller objects still fail at a reached marker.
        if !call[at..].starts_with('@') {
            continue;
        }
        let close =
            matching_paren(call, open).ok_or("unrecognized call signature in lifetime scan")?;
        let name = call[at + 1..open].trim_matches('"');
        let args = split(&call[open + 1..close]);
        let resolve = |a: &str| root(operand(a), f, &defs, &mut BTreeSet::new());
        if name.starts_with("llvm.lifetime.") {
            if !matches!(name, "llvm.lifetime.start.p0" | "llvm.lifetime.end.p0") {
                return Err(format!("unrecognized lifetime intrinsic {name}"));
            }
            let pointer = match args.as_slice() {
                [p] => *p,
                [_, p] => *p,
                _ => return Err(format!("unsupported intrinsic signature {name}: {args:?}")),
            };
            if typed(pointer)?.0 != "ptr" {
                return Err(format!("unsupported intrinsic signature {name}: {args:?}"));
            }
            plan.roots
                .push((resolve(pointer), name.starts_with("llvm.lifetime.start")));
        } else if module.defines(name) {
            plan.calls.push((
                name.into(),
                args.iter()
                    .map(|a| {
                        if typed(a).is_ok_and(|(t, _)| t == "ptr") {
                            resolve(a)
                        } else {
                            Root::Other
                        }
                    })
                    .collect(),
            ));
        }
    }
    Ok(plan)
}

/// Monotone fixed point: recursion never guesses a marker-free summary.
fn managed(
    module: &Module,
    entry: Rc<Function>,
) -> Result<BTreeMap<String, BTreeSet<String>>, String> {
    let mut functions = BTreeMap::new();
    let mut plans = BTreeMap::new();
    let mut pending = vec![entry];
    while let Some(f) = pending.pop() {
        if functions.contains_key(&f.name) {
            continue;
        }
        let plan = scan(&f, module)?;
        for (callee, args) in &plan.calls {
            if args
                .iter()
                .any(|r| matches!(r, Root::Stack(_) | Root::Parameter(_) | Root::Unknown))
            {
                pending.push(Rc::new(module.function(callee)?));
            }
        }
        plans.insert(f.name.clone(), plan);
        functions.insert(f.name.clone(), f);
    }
    let mut targets: BTreeMap<String, BTreeSet<String>> = functions
        .keys()
        .map(|n| (n.clone(), BTreeSet::new()))
        .collect();
    loop {
        let mut changed = false;
        for (name, plan) in &plans {
            let mut roots = plan.roots.clone();
            for (callee, args) in &plan.calls {
                if let Some(f) = functions.get(callee) {
                    for (i, (param, _)) in f.arguments.iter().enumerate() {
                        if targets[callee].contains(param) {
                            roots.push((args.get(i).cloned().unwrap_or(Root::Unknown), true));
                        }
                    }
                }
            }
            for (r, starts) in roots {
                match r {
                    Root::Stack(n) | Root::Parameter(n) => {
                        if starts {
                            changed |= targets.get_mut(name).unwrap().insert(n);
                        }
                    }
                    Root::Unknown => {
                        return Err(format!(
                            "unresolved lifetime association in {name} is unsupported"
                        ));
                    }
                    Root::Poison | Root::Other => {}
                }
            }
        }
        if !changed {
            return Ok(targets);
        }
    }
}

impl Engine {
    pub fn stack_managed(&mut self, state: &State, register: &str) -> Result<bool, String> {
        let key = format!("{}\0{}", state.module, state.function);
        if !self.lifetimes.contains_key(&key) {
            let f = self.current(state)?;
            let module = self
                .modules
                .iter()
                .find(|m| m.name == state.module)
                .cloned()
                .ok_or("lifetime module missing")?;
            for (name, targets) in managed(&module, f)? {
                self.lifetimes
                    .insert(format!("{}\0{name}", state.module), targets);
            }
        }
        Ok(self.lifetimes[&key].contains(register))
    }

    pub fn lifetime_call(
        &mut self,
        state: &mut State,
        callee: &str,
        args: &[&str],
    ) -> Result<Step, String> {
        if !matches!(callee, "llvm.lifetime.start.p0" | "llvm.lifetime.end.p0") {
            return Err(format!("unrecognized lifetime intrinsic {callee}"));
        }
        let (length, argument) = match args {
            [p] => (None, *p),
            [n, p] => (Some(*n), *p),
            _ => {
                return Err(format!(
                    "unsupported intrinsic signature {callee}: {args:?}"
                ));
            }
        };
        if typed(argument)?.0 != "ptr" {
            return Err(format!(
                "unsupported intrinsic signature {callee}: {args:?}"
            ));
        }
        // A marker accepts poison without observing it. Keep undef distinct:
        // observing undef would collapse it to definedness=false and could
        // incorrectly treat an unresolved lifetime operand as a poison no-op.
        let pointer = self.typed_transfer(state, argument)?;
        // No effect when poison; symbolic definedness is refused rather than
        // accidentally killing/reviving an object on the poison branch.
        if pointer.defined == "false" {
            return Ok(Step::Continue);
        }
        if pointer.defined != "true" {
            match self.solver.check(&state.constraints, &pointer.defined)? {
                Sat::No => return Ok(Step::Continue),
                Sat::Unknown => return Err("solver unknown resolving lifetime poison".into()),
                Sat::Yes => {
                    if self
                        .solver
                        .check(&state.constraints, &crate::value::not(&pointer.defined))?
                        != Sat::No
                    {
                        return Err("conditionally poison lifetime pointer is unsupported".into());
                    }
                }
            }
        }
        let pointer = self.resolve_pointer(state, &pointer)?;
        let (name, offset) = pointer.pointer()?;
        let object = state.memory.get(name).ok_or("unknown lifetime object")?;
        if !object.stack || !object.frame_live {
            return Err("non-stack or returned-frame lifetime pointer is unsupported".into());
        }
        if callee.starts_with("llvm.lifetime.start") && !object.managed {
            return Err("unresolved caller-frame lifetime association is unsupported".into());
        }
        if length.is_some_and(|n| {
            object
                .size
                .concrete()
                .is_none_or(|size| n != format!("i64 {size}"))
                && n != "i64 -1"
        }) {
            return Err("partial lifetimes are unsupported".into());
        }
        if self.solver.check(
            &state.constraints,
            &format!("(not (= {offset} {}))", bv(0, 64)),
        )? != Sat::No
        {
            return Err("non-base lifetime pointer is unsupported".into());
        }
        if callee.starts_with("llvm.lifetime.start") {
            let (base, size) = (object.base.clone(), object.size.term());
            for (other_name, other) in &state.memory {
                if other_name != name && other.alive && other.allocated {
                    let fact = format!(
                        "(or (bvule (bvadd {base} {}) {}) (bvule (bvadd {} {}) {base}))",
                        size,
                        other.base,
                        other.base,
                        other.size.term()
                    );
                    self.solver.facts.fact(&fact);
                    state.constraints.push(fact);
                }
            }
        }
        let object = state
            .memory
            .get_mut(name)
            .ok_or("unknown lifetime object")?;
        object.alive = callee.starts_with("llvm.lifetime.start");
        if object.alive {
            object.reset_flags();
        }
        Ok(Step::Continue)
    }
}
