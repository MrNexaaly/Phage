//! Bounded path exploration. Each state owns its SSA bindings and memory;
//! phi assignments read the predecessor environment simultaneously.
//! Feasible limits and unsupported instructions block successful verdicts.

use crate::{
    ir::{Function, Module, split, typed},
    memory::Memory,
    solver::{Sat, Solver},
    value::{Value, not, truth},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

mod state;
pub use state::{Engine, Frame, Library, State, Step, Verdict};

impl Engine {
    pub fn new(timeout_ms: u64, unwind: usize, max_states: usize) -> Result<Self, String> {
        Ok(Self {
            solver: Solver::new(timeout_ms)?,
            fresh: 0,
            inputs: Vec::new(),
            max_slice_len: None,
            buffers: Vec::new(),
            input_labels: Vec::new(),
            unwind,
            max_states,
            noreturn: BTreeSet::new(),
            modules: Vec::new(),
            functions: BTreeMap::new(),
            call_depth: 32,
            library: Library::Disabled,
            assumptions: BTreeSet::new(),
            global_keys: BTreeMap::new(),
            tainted: BTreeSet::new(),
            opaque_functions: BTreeSet::new(),
            definitions: std::collections::HashMap::new(),
            xor_forms: std::collections::HashMap::new(),
            random_sources: Vec::new(),
            simplifier: Default::default(),
            shapes: Default::default(),
            lifetimes: BTreeMap::new(),
        })
    }
    pub fn symbol(&mut self, sort: &str) -> Result<String, String> {
        let name = format!("v{}", self.fresh);
        self.fresh += 1;
        self.solver.declare(&name, sort)?;
        self.simplifier.declared(&name, sort);
        Ok(name)
    }
    fn violation(
        &mut self,
        state: &State,
        condition: &str,
        detail: &str,
        states: usize,
        completed: usize,
    ) -> Result<Option<Verdict>, String> {
        let (answer, exact) = self.exact_check(state, condition)?;
        match answer {
            Sat::No => Ok(None),
            Sat::Unknown => Ok(Some(Verdict {
                status: "unknown".into(),
                detail: format!(
                    "solver could not decide a safety obligation: {}",
                    self.solver.last_unknown
                ),
                model: String::new(),
                states,
                queries: self.solver.queries,
                completed,
            })),
            Sat::Yes => {
                let model = self.witness_model(state, &exact, condition)?;
                Ok(Some(Verdict {
                    status: "counterexample".into(),
                    detail: detail.into(),
                    model,
                    states,
                    queries: self.solver.queries,
                    completed,
                }))
            }
        }
    }
    pub fn safety(&mut self, state: &State, valid: &str, detail: &str) -> Result<(), String> {
        // Instruction-level obligations use this error payload to retain a
        // concrete model, rather than continuing after invalid memory use.
        let (answer, exact) = self.exact_check(state, &not(valid))?;
        match answer {
            Sat::No => Ok(()),
            Sat::Unknown => Err(format!(
                "solver unknown at memory obligation: {}",
                self.solver.last_unknown
            )),
            Sat::Yes => Err(format!(
                "COUNTEREXAMPLE:{detail}\n{}",
                self.witness_model(state, &exact, &not(valid))?
            )),
        }
    }
    /// Verifies one parsed function; calls resolve through `self.modules`.
    pub fn verify(&mut self, function: &Function) -> Result<Verdict, String> {
        let function = Rc::new(function.clone());
        self.functions.insert(
            format!("{}\u{0}{}", function.module, function.name),
            function.clone(),
        );
        self.noreturn.extend(function.noreturn.iter().cloned());
        let mut initial = State {
            function: function.name.clone(),
            module: function.module.clone(),
            block: function.entry.clone(),
            predecessor: String::new(),
            index: 0,
            env: BTreeMap::new(),
            memory: BTreeMap::new(),
            constraints: Vec::new(),
            visits: BTreeMap::new(),
            frames: Vec::new(),
            feasible: false,
            abstractions: Vec::new(),
            checked: 0,
            locals: Vec::new(),
        };
        self.entry_arguments(&function, &mut initial)?;
        // Constants exist from the start; mutable globals are allocated on
        // first reference, which is where their assumption is recorded.
        let module = self
            .modules
            .iter()
            .find(|m| m.name == function.module)
            .cloned();
        for global in module
            .iter()
            .flat_map(|m| m.globals.values())
            .filter(|g| !g.mutable)
        {
            let key = Self::qualify(&function.module, &global.name);
            if !initial.memory.contains_key(&key) {
                // One that cannot be allocated fails only where referenced.
                let _ = self.global(&mut initial, &function.module, global);
            }
        }
        let mut pending = vec![initial];
        let mut states = 0;
        let mut completed = 0;
        let mut unknown = Vec::new();
        while let Some(mut state) = pending.pop() {
            if let Some(verdict) = self.memory_limit(states, completed) {
                return Ok(verdict);
            }
            self.solver.context = format!("path entering block {}", state.block);
            if !state.feasible {
                match self.solver.check(&state.constraints, "true")? {
                    Sat::No => continue,
                    Sat::Unknown => {
                        unknown.push(format!(
                            "solver unknown on path feasibility: {}",
                            self.solver.last_unknown
                        ));
                        continue;
                    }
                    Sat::Yes => state.checked = state.constraints.len(),
                }
            }
            state.feasible = false;
            let function = self.current(&state)?;
            states += 1;
            if states > self.max_states {
                unknown.push("state exploration limit reached".into());
                break;
            }
            let key = format!("{}:{}", state.function, state.block);
            if state.index == 0 {
                let count = state.visits.entry(key).or_default();
                *count += 1;
                if *count > self.unwind {
                    unknown.push(format!(
                        "reachable block {} of {} exceeds visit bound {}",
                        state.block, state.function, self.unwind
                    ));
                    continue;
                }
            }
            let block = function
                .blocks
                .get(&state.block)
                .ok_or("branch target missing")?;
            if state.index == 0 {
                // Phi incomings are read from the predecessor environment
                // below, so the globals and literal-address objects they
                // name exist first, as for any other instruction.
                for line in block.iter().filter(|l| l.text.contains(" = phi ")) {
                    if line.text.contains('@') {
                        self.globals_for(&mut state, &line.text)?;
                    }
                    if line.text.contains("inttoptr (i64 ") {
                        self.constant_pointers(&mut state, &line.text);
                    }
                }
            }
            let previous = state.clone();
            let mut phi = Vec::new();
            for line in block.iter().filter(|_| state.index == 0) {
                if let Some((name, rest)) = line.text.split_once(" = phi ") {
                    let (ty, choices) = typed(rest)?;
                    let mut selected = None;
                    for choice in split(choices) {
                        let choice = choice.trim_start_matches('[').trim_end_matches(']');
                        let pair = split(choice);
                        if pair.len() == 2 && pair[1].trim_start_matches('%') == state.predecessor {
                            selected = Some(self.transfer_operand(&previous, pair[0], ty)?);
                        }
                    }
                    phi.push((
                        name.to_owned(),
                        selected.ok_or("phi predecessor is missing")?,
                    ));
                }
            }
            for (name, value) in phi {
                state.env.insert(name, value);
            }
            let mut terminated = false;
            let resume = std::mem::take(&mut state.index);
            for (position, line) in block.iter().enumerate().skip(resume) {
                if let Some(verdict) = self.memory_limit(states, completed) {
                    return Ok(verdict);
                }
                if line.text.contains(" = phi ") {
                    continue;
                }
                self.solver.context = format!("instruction at LLVM line {}", line.number);
                if crate::setting("TRACE_STEPS").is_some() {
                    eprintln!(
                        "step {} {} {}: {}",
                        state.module, state.block, line.number, line.text
                    );
                }
                let step = match self.execute(&mut state, line) {
                    Ok(step) => step,
                    Err(message) if message.starts_with(crate::pointers::FORK) => {
                        let condition = message[crate::pointers::FORK.len()..].to_owned();
                        for fact in [not(&condition), condition] {
                            let mut branch = state.clone();
                            branch.constraints.push(fact);
                            branch.index = position;
                            pending.push(branch);
                        }
                        terminated = true;
                        break;
                    }
                    Err(message) if message.starts_with("COUNTEREXAMPLE:") => {
                        let (detail, model) = message.split_once('\n').unwrap_or((&message, ""));
                        return Ok(Verdict {
                            status: "counterexample".into(),
                            detail: format!(
                                "{} at {}",
                                detail.trim_start_matches("COUNTEREXAMPLE:"),
                                Self::at(&state, line.number)
                            ),
                            model: model.into(),
                            states,
                            queries: self.solver.queries,
                            completed,
                        });
                    }
                    Err(message) => {
                        unknown.push(format!("{message} at {}", Self::at(&state, line.number)));
                        terminated = true;
                        break;
                    }
                };
                self.name_memory(&mut state)?;
                match step {
                    Step::Continue => {}
                    Step::Jump(target) => {
                        state.predecessor = state.block.clone();
                        state.block = target;
                        pending.push(state.clone());
                        terminated = true;
                        break;
                    }
                    Step::Branch(condition, yes, no) => {
                        if let Some(verdict) = self.violation(
                            &state,
                            &not(&condition.defined),
                            &format!(
                                "poison used as branch condition at {}",
                                Self::at(&state, line.number)
                            ),
                            states,
                            completed,
                        )? {
                            return Ok(verdict);
                        }
                        let condition = truth(&condition)?;
                        let condition = crate::ground::fold(&condition).unwrap_or(condition);
                        state.predecessor = state.block.clone();
                        // A literal condition takes one edge and adds no path
                        // condition; feasibility is inherited like a call's, and
                        // a completed path is rechecked before it counts.
                        if matches!(condition.as_str(), "true" | "false") {
                            state.block = if condition == "true" { yes } else { no };
                            state.feasible = true;
                            pending.push(state.clone());
                            terminated = true;
                            break;
                        }
                        let mut other = state.clone();
                        other.block = no;
                        other.constraints.push(not(&condition));
                        pending.push(other);
                        state.block = yes;
                        state.constraints.push(condition);
                        pending.push(state.clone());
                        terminated = true;
                        break;
                    }
                    Step::Switch(scrutinee, targets, default) => {
                        if let Some(verdict) = self.violation(
                            &state,
                            &not(&scrutinee.defined),
                            &format!(
                                "poison used as switch condition at {}",
                                Self::at(&state, line.number)
                            ),
                            states,
                            completed,
                        )? {
                            return Ok(verdict);
                        }
                        let mut misses = Vec::new();
                        let mut taken = None;
                        for (constant, target) in targets {
                            let hit = format!("(= {} {constant})", scrutinee.expr);
                            let hit = crate::ground::fold(&hit).unwrap_or(hit);
                            match hit.as_str() {
                                "false" => continue,
                                // A literal scrutinee takes exactly this case.
                                "true" => {
                                    taken = Some(target);
                                    break;
                                }
                                _ => {}
                            }
                            misses.push(not(&hit));
                            let mut next = state.clone();
                            next.predecessor = state.block.clone();
                            next.block = target;
                            next.constraints.push(hit);
                            pending.push(next);
                        }
                        state.predecessor = state.block.clone();
                        if let Some(target) = taken {
                            state.block = target;
                            state.feasible = true;
                        } else if misses.is_empty() {
                            // Every case is literally missed: the default is taken.
                            state.block = default;
                            state.feasible = true;
                        } else {
                            state.block = default;
                            state.constraints.push(crate::value::and(&misses));
                        }
                        pending.push(state.clone());
                        terminated = true;
                        break;
                    }
                    Step::Return(value) => {
                        if let Some(frame) = state.frames.pop() {
                            // A returning function's stack objects end their lifetime;
                            // pointers to them dangle.
                            for name in std::mem::replace(&mut state.locals, frame.locals) {
                                if let Some(object) = state.memory.get_mut(&name) {
                                    object.alive = false;
                                    object.frame_live = false;
                                }
                            }
                            // Violated return facts make the result poison;
                            // with noundef that is undefined behavior.
                            let attributes = format!(
                                "{} {}",
                                function.result_attributes, frame.result_attributes
                            );
                            let value = value
                                .map(|v| crate::calls::result_contract(&attributes, v))
                                .transpose()?;
                            if attributes.split_whitespace().any(|w| w == "noundef")
                                && let Some(value) = &value
                                && let Some(verdict) = self.violation(
                                    &state,
                                    &not(&crate::value::fully_defined(value)),
                                    &format!(
                                        "poison returned from noundef function {} at {}",
                                        function.name,
                                        Self::at(&state, line.number)
                                    ),
                                    states,
                                    completed,
                                )?
                            {
                                return Ok(verdict);
                            }
                            state.function = frame.function;
                            state.module = frame.module;
                            state.block = frame.block;
                            state.predecessor = frame.predecessor;
                            state.index = frame.index;
                            state.env = frame.env;
                            state.visits = frame.visits;
                            match (frame.destination, value) {
                                (Some(name), Some(value)) => {
                                    state.env.insert(name, value);
                                }
                                (Some(_), None) => return Err("void call result is used".into()),
                                _ => {}
                            }
                            state.feasible = true;
                            pending.push(state.clone());
                            terminated = true;
                            break;
                        }
                        // The property's result is observed (undef may be false).
                        let value =
                            crate::operands::observe(value.ok_or("entry function returned void")?)?;
                        if let Some(verdict) = self.violation(
                            &state,
                            &not(&value.defined),
                            &format!("poison returned at {}", Self::at(&state, line.number)),
                            states,
                            completed,
                        )? {
                            return Ok(verdict);
                        }
                        let condition = truth(&value)?;
                        if let Some(verdict) = self.violation(
                            &state,
                            &not(&condition),
                            &format!(
                                "property returned false at {}",
                                Self::at(&state, line.number)
                            ),
                            states,
                            completed,
                        )? {
                            return Ok(verdict);
                        }
                        if state.checked < state.constraints.len() {
                            match self.solver.check(&state.constraints, "true")? {
                                Sat::Yes => {}
                                Sat::No => {
                                    terminated = true;
                                    break;
                                }
                                Sat::Unknown => {
                                    unknown.push(format!(
                                        "solver unknown on path feasibility: {}",
                                        self.solver.last_unknown
                                    ));
                                    terminated = true;
                                    break;
                                }
                            }
                        }
                        completed += 1;
                        terminated = true;
                        break;
                    }
                    Step::Call(callee, arguments, destination, result_attributes) => {
                        if state.frames.len() >= self.call_depth {
                            unknown.push(format!(
                                "call depth limit {} reached at {}",
                                self.call_depth, callee.name
                            ));
                            terminated = true;
                            break;
                        }
                        if callee.arguments.len() != arguments.len() {
                            return Err(format!("argument count differs for {}", callee.name));
                        }
                        let env = callee
                            .arguments
                            .iter()
                            .map(|(name, _)| name.clone())
                            .zip(arguments)
                            .collect();
                        state.frames.push(Frame {
                            function: std::mem::replace(&mut state.function, callee.name.clone()),
                            module: std::mem::replace(&mut state.module, callee.module.clone()),
                            block: std::mem::replace(&mut state.block, callee.entry.clone()),
                            predecessor: std::mem::take(&mut state.predecessor),
                            index: position + 1,
                            env: std::mem::replace(&mut state.env, env),
                            destination,
                            locals: std::mem::take(&mut state.locals),
                            visits: std::mem::take(&mut state.visits),
                            result_attributes,
                        });
                        state.feasible = true;
                        pending.push(state.clone());
                        terminated = true;
                        break;
                    }
                    Step::Panic(detail) => {
                        if let Some(verdict) = self.violation(
                            &state,
                            "true",
                            &format!("{detail} at {}", Self::at(&state, line.number)),
                            states,
                            completed,
                        )? {
                            return Ok(verdict);
                        }
                        terminated = true;
                        break;
                    }
                }
            }
            if !terminated {
                unknown.push(format!("block {} has no supported terminator", state.block));
            }
        }
        let (status, detail) = if !unknown.is_empty() {
            ("unknown", unknown.join("; "))
        } else if completed == 0 {
            (
                "unknown",
                "no completed paths; refusing a vacuous proof".into(),
            )
        } else {
            (
                "proved",
                "all reachable paths within the explicit block bound completed".into(),
            )
        };
        Ok(Verdict {
            status: status.into(),
            detail,
            model: String::new(),
            states,
            queries: self.solver.queries,
            completed,
        })
    }
}

#[cfg(test)]
mod repomap_tests;
#[cfg(test)]
mod tests;
