//! Z3 process transport, reply deadlines and cancellation recovery.
use super::*;
use std::io::{BufRead, BufReader};

impl Solver {
    pub(super) fn configure(&mut self) -> Result<(), String> {
        self.write("(set-option :produce-models true)")?;
        // Fresh SSA symbols must survive backtracking across path frames.
        self.write("(set-option :global-decls true)")?;
        self.write(&format!("(set-option :timeout {})", self.timeout_ms))?;
        self.write(&format!("(set-logic {})", self.logic))?;
        for option in self.options.clone() {
            self.write(&option)?;
        }
        Ok(())
    }

    /// Cancels the pending incremental check. Z3 answers SIGINT during
    /// `check-sat` with `unknown` and keeps its declarations and assertion
    /// stack, so a racer's win or a deadline costs no replay of thousands of
    /// definitions (that replay made run times vary 3x between identical
    /// runs). Between commands SIGINT ends Z3, so success is confirmed by an
    /// semantic check round trip; any other outcome restarts.
    pub(super) fn interrupt(&mut self) -> Result<(), String> {
        let signalled = Command::new("kill")
            .args(["-INT", &self.child.id().to_string()])
            .status()
            .is_ok_and(|s| s.success());
        let mut stash = Vec::new();
        let mut wait = |solver: &mut Solver| {
            let deadline = Instant::now() + Duration::from_millis(INTERRUPT_MS);
            solver.line_until(deadline, &mut stash).ok().flatten()
        };
        if signalled
            && wait(self).is_some_and(|a| matches!(a.as_str(), "sat" | "unsat" | "unknown"))
            && self.write("(pop 1)").is_ok()
            && self.write("(echo \"ru-sync\")").is_ok()
            && wait(self).is_some_and(|l| l.trim_matches('"') == "ru-sync")
            // Echo does not touch the solver. A poisoned context may echo
            // successfully and fail at the very next push.
            && self.recovery_probe().is_ok()
            && wait(self).is_some_and(|a| a == "unsat")
            && self.write("(pop 1)").is_ok()
        {
            self.interrupts += 1;
            return Ok(());
        }
        self.restart()
    }

    /// Exercise the solver stack without solving the path again.
    fn recovery_probe(&mut self) -> Result<(), String> {
        self.write("(push 1)")?;
        self.write("(assert false)")?;
        self.assertions += 1;
        self.write("(check-sat)")?;
        self.checks += 1;
        Ok(())
    }

    /// Floating point needs a logic with FloatingPoint (all three solvers
    /// accept `ALL`); switched once, with a restart, so programs without
    /// floats keep the bit-vector logic and its solver configuration.
    pub fn enable_floats(&mut self) -> Result<(), String> {
        if self.logic == "ALL" {
            return Ok(());
        }
        self.logic = "ALL";
        self.restart()
    }

    /// Replaces a busy incremental process and replays its declarations.
    pub(super) fn restart(&mut self) -> Result<(), String> {
        self.restarts += 1;
        let _ = self.child.kill();
        let _ = self.child.wait();
        self.generation += 1;
        let (child, input) = spawn_z3(&self.sender, self.generation)?;
        self.child = child;
        self.input = input;
        self.active.clear();
        self.configure()?;
        for declaration in self.declarations.clone() {
            self.write(&declaration)?;
        }
        Ok(())
    }

    pub(super) fn write(&mut self, line: &str) -> Result<(), String> {
        writeln!(self.input, "{line}").map_err(|e| e.to_string())?;
        self.input.flush().map_err(|e| e.to_string())
    }

    /// Sends a raw command; options are kept for replay after a restart.
    #[cfg(test)]
    pub fn send(&mut self, line: &str) -> Result<(), String> {
        if line.starts_with("(set-option") {
            self.options.push(line.to_owned());
        }
        self.write(line)
    }

    /// Reads a get-value reply; `None` when the deadline passes first.
    pub(super) fn get_value(
        &mut self,
        values: &str,
        deadline: Instant,
        stash: &mut Vec<Event>,
    ) -> Result<Option<String>, String> {
        self.write(&format!("(get-value ({values}))"))?;
        let mut result = String::new();
        let mut balance = 0isize;
        loop {
            let Some(line) = self.line_until(deadline, stash)? else {
                return Ok(None);
            };
            balance += line.bytes().filter(|b| *b == b'(').count() as isize;
            balance -= line.bytes().filter(|b| *b == b')').count() as isize;
            result.push_str(&line);
            result.push('\n');
            if balance <= 0 {
                return Ok(Some(result));
            }
        }
    }

    /// The next incremental line before `deadline`, stashing racer events.
    pub(super) fn line_until(
        &mut self,
        deadline: Instant,
        stash: &mut Vec<Event>,
    ) -> Result<Option<String>, String> {
        loop {
            let Some(wait) = deadline.checked_duration_since(Instant::now()) else {
                return Ok(None);
            };
            self.budget.check()?;
            match self
                .events
                .recv_timeout(wait.min(Duration::from_millis(20)))
            {
                Ok(Event::Line(generation, line)) if generation == self.generation => {
                    return self.exited(line).map(Some);
                }
                Ok(event @ Event::Race(..)) => stash.push(event),
                Ok(Event::Line(..)) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                Err(_) => return Err("solver channel closed".into()),
            }
        }
    }

    /// The next line of the current incremental process.
    #[cfg(test)]
    pub(super) fn line(&mut self) -> Result<String, String> {
        loop {
            match self.events.recv() {
                Ok(Event::Line(generation, line)) if generation == self.generation => {
                    return self.exited(line);
                }
                Ok(_) => {}
                Err(_) => return Err("solver channel closed".into()),
            }
        }
    }

    pub(super) fn exited(&mut self, line: Option<String>) -> Result<String, String> {
        match line {
            Some(line) => Ok(line),
            None => {
                let status = self
                    .child
                    .try_wait()
                    .map_err(|e| e.to_string())?
                    .map_or("exit status unavailable".into(), |s| s.to_string());
                Err(format!("Z3 exited or closed output: {status}"))
            }
        }
    }
}

pub(super) fn spawn_z3(
    sender: &Sender<Event>,
    generation: u64,
) -> Result<(Child, ChildStdin), String> {
    let mut child = Command::new("z3")
        .args(["-in", "-smt2"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| format!("cannot start Z3: {e}"))?;
    let input = child.stdin.take().ok_or("Z3 stdin missing")?;
    let output = BufReader::new(child.stdout.take().ok_or("Z3 stdout missing")?);
    let sender = sender.clone();
    std::thread::spawn(move || {
        for line in output.lines() {
            let Ok(line) = line else { break };
            if sender
                .send(Event::Line(generation, Some(line.trim().to_owned())))
                .is_err()
            {
                return;
            }
        }
        let _ = sender.send(Event::Line(generation, None));
    });
    Ok((child, input))
}
