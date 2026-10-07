//! Stateful SMT-LIB connection to Z3. Every query checks the current path
//! under push/pop; unknown is propagated rather than interpreted as unsat.
//! Path prefixes stay asserted between queries; declarations are global.
//!
//! Incremental Z3 is fast for the many small queries of path exploration
//! but skips its bit-vector preprocessing, so hard queries can be several
//! times slower than a fresh process. A query still running after a short
//! stage is raced as a standalone script against a fresh Z3 and cvc5 with
//! int-blasting (which alone decides some nonlinear arithmetic). The first
//! definite answer wins; if a racer wins, the busy incremental process is
//! replaced and its declarations replayed, so its pipe never desynchronizes.

use std::{
    io::{Read, Write},
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc::{Receiver, Sender, channel},
    time::{Duration, Instant},
};

/// How long the incremental solver runs alone before racers start.
/// Longest wait before racing; the actual wait adapts (`stage_ms`).
const STAGE_MS: u64 = 100;
/// How long a SIGINT-cancelled Z3 may take to answer before a restart.
const INTERRUPT_MS: u64 = 3000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sat {
    Yes,
    No,
    Unknown,
}

enum Event {
    /// A line from the incremental Z3 of a generation; `None` at exit.
    Line(u64, Option<String>),
    /// Complete output of one racer in a race.
    Race(u64, usize, String),
}

pub struct Solver {
    /// How long a query runs incrementally before racers start: halved
    /// each time a racer wins, grown back each time the incremental solver
    /// wins a race, so a workload whose queries keep going to racers stops
    /// paying the wait, while one that rarely races spawns no processes.
    stage_ms: u64,
    /// Allocation facts and what reaches them (slice.rs).
    pub facts: slice::Facts,
    pub budget: crate::budget::Budget,
    /// SMT-LIB logic: `QF_AUFBV`, or `ALL` once floating point is used.
    logic: &'static str,
    child: Child,
    input: ChildStdin,
    events: Receiver<Event>,
    sender: Sender<Event>,
    generation: u64,
    race: u64,
    /// Options sent after startup, replayed after a restart.
    options: Vec<String>,
    /// Racing may be disabled to model a solver that can decide nothing.
    pub portfolio: bool,
    pub queries: usize,
    declarations: Vec<String>,
    pub counterexample: String,
    active: Vec<String>,
    pub assertions: usize,
    /// Checks cancelled in place, and full restarts with replay.
    pub interrupts: usize,
    pub restarts: usize,
    pub checks: usize,
    pub context: String,
    pub slowest_context: String,
    pub slowest_seconds: f64,
    pub slowest_query: String,
    pub last_unknown: String,
    pub elapsed: Duration,
    timeout_ms: u64,
    /// Queries raced, and how many a racer decided first.
    pub escalations: usize,
    pub escalations_decided: usize,
}

impl Solver {
    pub fn new(timeout_ms: u64) -> Result<Self, String> {
        let (sender, events) = channel();
        let (child, input) = spawn_z3(&sender, 0)?;
        let mut solver = Self {
            stage_ms: STAGE_MS,
            facts: Default::default(),
            budget: crate::budget::Budget::new(crate::budget::default_mib()),
            logic: "QF_AUFBV",
            child,
            input,
            events,
            sender,
            generation: 0,
            race: 0,
            options: Vec::new(),
            portfolio: true,
            queries: 0,
            declarations: Vec::new(),
            counterexample: String::new(),
            active: Vec::new(),
            assertions: 0,
            interrupts: 0,
            restarts: 0,
            checks: 0,
            context: String::new(),
            slowest_context: String::new(),
            slowest_seconds: 0.0,
            slowest_query: String::new(),
            last_unknown: String::new(),
            elapsed: Duration::ZERO,
            timeout_ms,
            escalations: 0,
            escalations_decided: 0,
        };
        solver.configure()?;
        Ok(solver)
    }

    /// Defines `name` as an abbreviation of `body` (`define-fun`, a macro the
    /// solver expands while parsing: no new constraint, only shorter text).
    pub fn define(&mut self, name: &str, sort: &str, body: &str) -> Result<(), String> {
        self.facts.defined(name, body);
        let definition = format!("(define-fun {name} () {sort} {body})");
        self.declarations.push(definition.clone());
        self.write(&definition)
    }
    /// Declares an uninterpreted function; replayed like any declaration.
    pub fn declare_function(
        &mut self,
        name: &str,
        domain: &[String],
        range: &str,
    ) -> Result<(), String> {
        let declaration = format!("(declare-fun {name} ({}) {range})", domain.join(" "));
        self.declarations.push(declaration.clone());
        self.write(&declaration)
    }
    pub fn declare(&mut self, name: &str, sort: &str) -> Result<(), String> {
        let declaration = format!("(declare-fun {name} () {sort})");
        self.declarations.push(declaration.clone());
        self.write(&declaration)
    }

    fn activate(&mut self, constraints: &[String]) -> Result<(), String> {
        let common = self
            .active
            .iter()
            .zip(constraints)
            .take_while(|(a, b)| a == b)
            .count();
        let remove = self.active.len() - common;
        if remove > 0 {
            self.write(&format!("(pop {remove})"))?;
        }
        self.active.truncate(common);
        for constraint in &constraints[common..] {
            self.write("(push 1)")?;
            self.write(&format!("(assert {constraint})"))?;
            self.assertions += 1;
            self.active.push(constraint.clone());
        }
        Ok(())
    }

    pub fn check(&mut self, constraints: &[String], extra: &str) -> Result<Sat, String> {
        let sliced = self.facts.slice(constraints, extra);
        let constraints = sliced.as_deref().unwrap_or(constraints);
        let started = Instant::now();
        self.queries += 1;
        // A literal (or ground, hence foldable) false obligation is
        // unsatisfiable under every path.
        if extra == "false" || crate::ground::fold(extra).as_deref() == Some("false") {
            return Ok(Sat::No);
        }
        let (status, _) = self.decide(constraints, extra, None)?;
        let elapsed = started.elapsed();
        self.elapsed += elapsed;
        if let Some(dir) = crate::setting("DUMP_QUERIES")
            && elapsed.as_secs_f64() > 0.05
        {
            let path = std::path::Path::new(&dir).join(format!(
                "q{}-{:.3}.smt2",
                self.queries,
                elapsed.as_secs_f64()
            ));
            let _ = std::fs::write(path, self.query_text(constraints, extra));
        }
        if crate::setting("TRACE_QUERIES").is_some() {
            eprintln!(
                "query {:.4} {status:?} {} | {}",
                elapsed.as_secs_f64(),
                constraints.len(),
                self.context
            );
        }
        if elapsed.as_secs_f64() > self.slowest_seconds {
            self.slowest_seconds = elapsed.as_secs_f64();
            self.slowest_context = self.context.clone();
            self.slowest_query = self.query_text(constraints, extra);
        }
        Ok(status)
    }

    /// A model value of `expr` on the current path, if the path is satisfiable.
    pub fn value(&mut self, constraints: &[String], expr: &str) -> Result<Option<String>, String> {
        let started = Instant::now();
        let (status, reply) = self.decide(constraints, "true", Some(expr))?;
        self.elapsed += started.elapsed();
        if status != Sat::Yes {
            return Ok(None);
        }
        // The value is the last literal of `((expr value))`.
        Ok(reply
            .rsplit(|c: char| c.is_whitespace() || c == '(' || c == ')')
            .find(|t| t.starts_with("#x") || t.starts_with("#b"))
            .map(str::to_owned))
    }

    fn query_text(&self, constraints: &[String], extra: &str) -> String {
        format!(
            "(set-option :produce-models true)\n(set-logic {})\n{}\n{}\n(assert {extra})\n(check-sat)\n",
            self.logic,
            self.declarations.join("\n"),
            constraints
                .iter()
                .map(|c| format!("(assert {c})"))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    pub fn model(
        &mut self,
        constraints: &[String],
        extra: &str,
        inputs: &[String],
    ) -> Result<String, String> {
        let sliced = self.facts.slice(constraints, extra);
        let constraints = sliced.as_deref().unwrap_or(constraints);
        self.counterexample = self.query_text(constraints, extra);
        if inputs.is_empty() {
            return Ok("no symbolic arguments\n".into());
        }
        self.counterexample
            .push_str(&format!("(get-value ({}))\n", inputs.join(" ")));
        let started = Instant::now();
        let (status, values) = self.decide(constraints, extra, Some(&inputs.join(" ")))?;
        self.elapsed += started.elapsed();
        match status {
            Sat::Yes => Ok(values),
            _ => Err("counterexample changed during replay query".into()),
        }
    }

    /// Decides `constraints ∧ extra`, returning requested values when sat.
    fn decide(
        &mut self,
        constraints: &[String],
        extra: &str,
        values: Option<&str>,
    ) -> Result<(Sat, String), String> {
        self.budget.check()?;
        self.last_unknown.clear();
        self.activate(constraints)?;
        self.write("(push 1)")?;
        self.write(&format!("(assert {extra})"))?;
        self.assertions += 1;
        self.write("(check-sat)")?;
        self.checks += 1;
        let now = Instant::now();
        let deadline = now
            .checked_add(Duration::from_millis(self.timeout_ms))
            .unwrap_or(now + Duration::from_secs(86_400));
        let stage = now + Duration::from_millis(self.stage_ms);
        let mut racers = Racers(Vec::new());
        // Racer results that arrive while reading an incremental reply.
        let mut stash: Vec<Event> = Vec::new();
        let mut raced = false;
        let mut finished = 0;
        let mut incremental_done = false;
        let mut outcome = (Sat::Unknown, String::new());
        loop {
            self.budget.check()?;
            let now = Instant::now();
            if now >= deadline {
                break;
            }
            let wait = if !raced && self.portfolio && !incremental_done {
                stage.saturating_duration_since(now)
            } else {
                deadline - now
            };
            let event = if let Some(event) = stash.pop() {
                Some(event)
            } else {
                match self
                    .events
                    .recv_timeout(wait.min(Duration::from_millis(20)))
                {
                    Ok(event) => Some(event),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => None,
                    Err(_) => return Err("solver channel closed".into()),
                }
            };
            match event {
                None if !raced
                    && self.portfolio
                    && !incremental_done
                    && Instant::now() >= stage =>
                {
                    racers.0 = self.start_race(constraints, extra, values);
                    raced = true;
                }
                None => continue,
                Some(Event::Line(generation, _)) if generation != self.generation => {}
                Some(Event::Line(_, line)) => {
                    let answer = self.exited(line)?;
                    if answer.starts_with("(error") && answer.contains("canceled") {
                        self.last_unknown = format!(
                            "solver cancelled during context update on query {}",
                            self.queries
                        );
                        self.restart()?;
                        incremental_done = true;
                        break;
                    }
                    let status = match answer.as_str() {
                        "sat" => Sat::Yes,
                        "unsat" => Sat::No,
                        "unknown" => {
                            self.write("(get-info :reason-unknown)")?;
                            match self.line_until(deadline, &mut stash)? {
                                Some(reason) => {
                                    self.last_unknown = if reason.contains("resource")
                                        || reason.contains("rlimit")
                                    {
                                        format!(
                                            "solver resource limit on query {}: {reason}",
                                            self.queries
                                        )
                                    } else if reason.contains("timeout")
                                        || reason.contains("canceled") && Instant::now() >= deadline
                                    {
                                        self.timeout_detail()
                                    } else if reason.contains("canceled") {
                                        format!(
                                            "solver cancelled on query {}: {reason}",
                                            self.queries
                                        )
                                    } else {
                                        format!(
                                            "solver unknown on query {}: {reason}",
                                            self.queries
                                        )
                                    };
                                }
                                None => break,
                            }
                            Sat::Unknown
                        }
                        other => return Err(format!("Z3 returned {other}")),
                    };
                    let reply = match (status, values) {
                        (Sat::Yes, Some(values)) => {
                            match self.get_value(values, deadline, &mut stash)? {
                                Some(reply) => reply,
                                None => break,
                            }
                        }
                        _ => String::new(),
                    };
                    if status == Sat::Unknown {
                        // Timeout/cancellation can leave push/pop unusable.
                        self.restart()?;
                    } else {
                        self.write("(pop 1)")?;
                    }
                    incremental_done = true;
                    if status != Sat::Unknown {
                        if raced {
                            self.stage_ms = (self.stage_ms * 2 + 5).min(STAGE_MS);
                        }
                        outcome = (status, reply);
                        break;
                    }
                    if !raced && self.portfolio {
                        racers.0 = self.start_race(constraints, extra, values);
                        raced = true;
                    }
                    if finished == racers.0.len() {
                        break;
                    }
                }
                Some(Event::Race(race, _, _)) if race != self.race => {}
                Some(Event::Race(_, index, text)) => {
                    finished += 1;
                    let mut lines = text.lines();
                    let status = match lines.next().map(str::trim) {
                        Some("sat") => Sat::Yes,
                        Some("unsat") => Sat::No,
                        _ if incremental_done && finished == racers.0.len() => break,
                        _ => continue,
                    };
                    self.escalations_decided += 1;
                    self.stage_ms /= 2;
                    if crate::setting("TRACE_QUERIES").is_some() {
                        eprintln!("race won by {} ({status:?})", RACERS[index].0);
                    }
                    self.last_unknown = format!("decided by {} after racing", RACERS[index].0);
                    outcome = (status, lines.collect::<Vec<_>>().join("\n") + "\n");
                    if !incremental_done {
                        self.interrupt()?;
                        incremental_done = true;
                    }
                    break;
                }
            }
        }
        drop(racers);
        // Advance the race id so late racer output is ignored.
        self.race += 1;
        if !incremental_done {
            self.last_unknown = self.timeout_detail();
            self.interrupt()?;
        }
        if outcome.0 == Sat::Unknown && self.last_unknown.is_empty() {
            self.last_unknown = self.timeout_detail();
        }
        Ok(outcome)
    }

    fn timeout_detail(&self) -> String {
        format!(
            "solver timeout after {} ms on query {}",
            self.timeout_ms, self.queries
        )
    }

    /// Starts every available racer on a standalone script.
    fn start_race(
        &mut self,
        constraints: &[String],
        extra: &str,
        values: Option<&str>,
    ) -> Vec<Child> {
        self.escalations += 1;
        if crate::setting("TRACE_QUERIES").is_some() {
            eprintln!("race started: {}", self.context);
        }
        let mut script = self.query_text(constraints, extra);
        if let Some(values) = values {
            script.push_str(&format!("(get-value ({values}))\n"));
        }
        let mut children = Vec::new();
        for (index, (program, args)) in RACERS.iter().enumerate() {
            let args = args.iter().map(|a| {
                a.replace("{ms}", &self.timeout_ms.to_string())
                    .replace("{s}", &self.timeout_ms.div_ceil(1000).max(1).to_string())
            });
            let Ok(mut child) = Command::new(program)
                .args(args)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
            else {
                continue;
            };
            // Written from its own thread: a racer that stops reading must not
            // stall the coordinator, whose deadline still applies.
            if let Some(mut input) = child.stdin.take() {
                let script = script.clone();
                std::thread::spawn(move || {
                    let _ = input.write_all(script.as_bytes());
                });
            }
            let Some(mut output) = child.stdout.take() else {
                let _ = child.kill();
                continue;
            };
            let (sender, race) = (self.sender.clone(), self.race);
            std::thread::spawn(move || {
                let mut text = String::new();
                let _ = output.read_to_string(&mut text);
                let _ = sender.send(Event::Race(race, index, text));
            });
            children.push(child);
        }
        children
    }
}

/// Kills and reaps racers on every exit path, including errors.
struct Racers(Vec<Child>);

impl Drop for Racers {
    fn drop(&mut self) {
        for racer in &mut self.0 {
            let _ = racer.kill();
            let _ = racer.wait();
        }
    }
}

/// Racers: `{s}` and `{ms}` expand to the query time limit. A missing
/// binary simply does not race. Bitwuzla's bit-blaster decides hash-heavy
/// bit-vector queries (SipHash with unknown keys) several times faster.
const RACERS: [(&str, &[&str]); 3] = [
    ("z3", &["-in", "-smt2", "-T:{s}"]),
    ("bitwuzla", &["--lang", "smt2", "-m", "-t", "{ms}"]),
    (
        "cvc5",
        &[
            "--lang=smt2",
            "--solve-bv-as-int=sum",
            "--arrays-exp",
            "--tlimit={ms}",
        ],
    ),
];

impl Drop for Solver {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

mod transport;
use transport::spawn_z3;
mod slice;

#[cfg(test)]
mod tests;
