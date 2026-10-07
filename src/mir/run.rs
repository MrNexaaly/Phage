//! Path exploration with real call frames, caller-visible references and
//! recursion limits. A feasible incomplete path always prevents proof.

use super::{Address, Frame, Local, Node, State, Verifier, integer, parse::Function};
use crate::{
    engine::Verdict,
    solver::Sat,
    value::{Value, bv, not, truth},
};
use std::collections::BTreeMap;

pub enum Step {
    Next,
    Jump(String),
    Switch(Node, Vec<(String, String)>, String),
    Assert(Node, bool, String, String),
    Call(String, Vec<Node>, Address, String),
    Return,
    Panic(String),
}

impl Verifier {
    pub fn frame(
        &mut self,
        state: &mut State,
        function: &Function,
        args: Vec<Node>,
        bindings: BTreeMap<String, String>,
        resume: Option<(Address, String)>,
    ) -> Result<(), String> {
        if args.len() != function.params.len() {
            return Err("MIR argument count differs from callee".into());
        }
        let id = self.next_frame;
        self.next_frame += 1;
        state.frames.push(Frame {
            id,
            function: function.key.clone(),
            block: "bb0".into(),
            bindings,
            resume,
            visits: BTreeMap::new(),
        });
        for (local, ty) in &function.locals {
            let ty = state.ty(ty);
            let alive = !function.storage.contains(local)
                || local == "_0"
                || function.params.iter().any(|(n, _)| n == local);
            state.locals.insert(
                (id, local.clone()),
                Local {
                    value: Node::uninit(&ty),
                    alive,
                    generation: 0,
                },
            );
        }
        for ((local, ty), value) in function.params.iter().zip(args) {
            let expected = state.ty(ty);
            self.compatible(&expected, &value.ty)?;
            state
                .locals
                .get_mut(&(id, local.clone()))
                .ok_or("MIR argument local missing")?
                .value = value;
        }
        Ok(())
    }
    pub fn compatible(&self, expected: &str, actual: &str) -> Result<(), String> {
        let canonical = |ty: &str| {
            ty.replace("std::option::", "core::option::")
                .replace("std::result::", "core::result::")
                .replace(' ', "")
        };
        if canonical(expected) == canonical(actual) {
            Ok(())
        } else {
            Err(format!(
                "unsupported MIR type mismatch: {actual} into {expected}"
            ))
        }
    }
    pub fn check(&mut self, entry: &str) -> Result<Verdict, String> {
        let function = self.program.entry(entry)?.clone();
        if function.result != "bool" {
            return Err("MIR property must return bool".into());
        }
        let mut state = State {
            frames: Vec::new(),
            locals: BTreeMap::new(),
            constraints: Vec::new(),
        };
        let mut args = Vec::new();
        for (i, (local, ty)) in function.params.iter().enumerate() {
            let (width, _) =
                integer(ty).ok_or_else(|| format!("unsupported entry argument: {ty}"))?;
            let input = format!("input{i}");
            self.core
                .solver
                .declare(&input, &format!("(_ BitVec {width})"))?;
            self.core.inputs.push(input.clone());
            self.core
                .input_labels
                .push(function.labels.get(local).unwrap_or(local).clone());
            if ty == "char" {
                state.constraints.push(format!(
                    "(and (bvule {input} {}) (or (bvult {input} {}) (bvugt {input} {})))",
                    bv(0x10ffff, 32),
                    bv(0xd800, 32),
                    bv(0xdfff, 32)
                ));
            }
            args.push(Node::scalar(ty, Value::bits(input, width, "true".into())));
        }
        self.frame(&mut state, &function, args, BTreeMap::new(), None)?;
        let mut pending = vec![state];
        let (mut states, mut completed) = (0, 0);
        let mut unknown = Vec::new();
        while let Some(mut state) = pending.pop() {
            if let Some(verdict) = self.core.memory_limit(states, completed) {
                return Ok(verdict);
            }
            self.core.solver.context = format!(
                "MIR path entering {}::{}",
                state.frame()?.function,
                state.frame()?.block
            );
            match self.core.solver.check(&state.constraints, "true")? {
                Sat::No => continue,
                Sat::Unknown => {
                    unknown.push(format!("solver unknown: {}", self.core.solver.last_unknown));
                    continue;
                }
                Sat::Yes => {}
            }
            states += 1;
            if states > self.core.max_states {
                unknown.push("state exploration limit reached".into());
                break;
            }
            let frame = state.frame_mut()?;
            let count = frame.visits.entry(frame.block.clone()).or_default();
            *count += 1;
            if *count > self.core.unwind {
                unknown.push(format!(
                    "reachable MIR block {} exceeds visit bound {}",
                    frame.block, self.core.unwind
                ));
                continue;
            }
            let function = self
                .program
                .functions
                .get(&state.frame()?.function)
                .ok_or("MIR callee missing")?
                .clone();
            let block = function
                .blocks
                .get(&state.frame()?.block)
                .ok_or("MIR branch target missing")?;
            let mut terminated = false;
            for line in block {
                if let Some(verdict) = self.core.memory_limit(states, completed) {
                    return Ok(verdict);
                }
                self.core.solver.context = format!("MIR instruction at MIR line {}", line.number);
                let step = match self.statement(&mut state, line) {
                    Ok(step) => step,
                    Err(error) => {
                        if let Some(result) = self.failure(&error, line.number, states, completed) {
                            return Ok(result);
                        }
                        unknown.push(format!("{error} at MIR line {}", line.number));
                        terminated = true;
                        break;
                    }
                };
                if matches!(&step, Step::Next) {
                    continue;
                }
                let transition = (|| -> Result<(), String> {
                    match step {
                        Step::Next => return Ok(()),
                        Step::Jump(block) => {
                            state.frame_mut()?.block = block;
                            pending.push(state.clone());
                        }
                        Step::Switch(value, choices, otherwise) => {
                            self.observe(&state, &value)?;
                            let scalar = value.value()?;
                            let mut exclusions = Vec::new();
                            for (number, target) in choices {
                                let number = crate::value::constant(&number, scalar.width()?)?;
                                let condition = format!("(= {} {})", scalar.expr, number.expr);
                                let mut branch = state.clone();
                                branch.constraints.push(condition.clone());
                                branch.frame_mut()?.block = target;
                                pending.push(branch);
                                exclusions.push(not(&condition));
                            }
                            state.constraints.extend(exclusions);
                            state.frame_mut()?.block = otherwise;
                            pending.push(state.clone());
                        }
                        Step::Assert(value, expected, message, target) => {
                            self.observe(&state, &value)?;
                            let valid = truth(value.value()?)?;
                            let valid = if expected { valid } else { not(&valid) };
                            self.obligation(
                                &state,
                                &valid,
                                &format!("Rust assertion failed: {message}"),
                            )?;
                            state.constraints.push(valid);
                            state.frame_mut()?.block = target;
                            pending.push(state.clone());
                        }
                        Step::Call(name, args, destination, target) => {
                            if let Some((function, bindings)) =
                                self.resolve(&state, &name, &args)?
                            {
                                if state.frames.len() >= self.depth {
                                    return Err(format!(
                                        "MIR call depth limit {} reached at {name}",
                                        self.depth
                                    ));
                                }
                                self.frame(
                                    &mut state,
                                    &function,
                                    args,
                                    bindings,
                                    Some((destination, target)),
                                )?;
                                pending.push(state.clone());
                            } else {
                                let value = self.model(&mut state, &name, args)?;
                                if destination.projections.is_empty() {
                                    let expected = &state
                                        .locals
                                        .get(&(destination.frame, destination.local.clone()))
                                        .ok_or("MIR caller return destination missing")?
                                        .value
                                        .ty;
                                    self.compatible(expected, &value.ty)?;
                                }
                                self.write_address(&mut state, &destination, value)?;
                                state.frame_mut()?.block = target;
                                pending.push(state.clone());
                            }
                        }
                        Step::Return => {
                            let value = self.read_place(&mut state, "_0", true)?;
                            let frame = state.frames.pop().ok_or("MIR return frame missing")?;
                            for ((id, _), slot) in &mut state.locals {
                                if *id == frame.id {
                                    slot.alive = false;
                                }
                            }
                            if let Some((destination, target)) = frame.resume {
                                if destination.projections.is_empty() {
                                    let expected = &state
                                        .locals
                                        .get(&(destination.frame, destination.local.clone()))
                                        .ok_or("MIR caller return destination missing")?
                                        .value
                                        .ty;
                                    self.compatible(expected, &value.ty)?;
                                }
                                self.write_address(&mut state, &destination, value)?;
                                state.frame_mut()?.block = target;
                                pending.push(state.clone());
                            } else {
                                self.observe(&state, &value)?;
                                self.obligation(
                                    &state,
                                    &truth(value.value()?)?,
                                    "property returned false",
                                )?;
                                completed += 1;
                            }
                        }
                        Step::Panic(message) => self.obligation(&state, "false", &message)?,
                    }
                    Ok(())
                })();
                if let Err(error) = transition {
                    if let Some(result) = self.failure(&error, line.number, states, completed) {
                        return Ok(result);
                    }
                    unknown.push(format!("{error} at MIR line {}", line.number));
                }
                // All steps except Next are terminators. Calls never execute
                // their caller successor before the callee returns.
                terminated = true;
                break;
            }
            if !terminated {
                unknown.push("MIR block has no supported terminator".into());
            }
        }
        let (status, detail) = if !unknown.is_empty() {
            ("unknown", unknown.join("; "))
        } else if completed == 0 {
            (
                "unknown",
                "no completed MIR paths; refusing a vacuous proof".into(),
            )
        } else {
            (
                "proved",
                "all reachable built-MIR paths completed within the explicit limits".into(),
            )
        };
        Ok(Verdict {
            status: status.into(),
            detail,
            model: String::new(),
            states,
            queries: self.core.solver.queries,
            completed,
        })
    }
    fn failure(
        &self,
        error: &str,
        line: usize,
        states: usize,
        completed: usize,
    ) -> Option<Verdict> {
        let error = error.strip_prefix("COUNTEREXAMPLE:")?;
        let (detail, model) = error.split_once('\n').unwrap_or((error, ""));
        Some(Verdict {
            status: "counterexample".into(),
            detail: format!("{detail} at MIR line {line}"),
            model: model.into(),
            states,
            queries: self.core.solver.queries,
            completed,
        })
    }
}
