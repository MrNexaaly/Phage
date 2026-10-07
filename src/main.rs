//! CLI and checked Rust-to-LLVM frontend. Results refer to the retained
//! compiled artifact and explicit exploration limits, never to all Rust.

/// Prints one line of the human summary. The report is already on disk and
/// the exit code is the verdict, so a closed stdout (`phage check x | head`)
/// must not turn either into a panic.
macro_rules! say {
    ($($arg:tt)*) => {{
        use std::io::Write as _;
        let _ = writeln!(std::io::stdout(), $($arg)*);
    }};
}

mod assume;
mod atomics;
mod budget;
mod bytes;
mod cache;
mod calls;
mod canonical;
mod constants;
mod debug;
mod diagnostics;
mod engine;
mod entry;
mod environment;
mod execute;
mod floats;
mod ground;
mod heap;
mod intrinsics;
mod ir;
mod knownbits;
mod layout;
mod library;
mod lifetime;
mod loads;
mod memory;
mod mir;
mod opaque;
mod operands;
mod partial;
mod pointers;
mod program;
mod report;
mod solver;
mod symbols;
mod toolchain;
mod value;
mod vectors;
use std::{
    path::PathBuf,
    process::{Command, ExitCode},
    time::Instant,
};

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("unknown: {error}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<u8, String> {
    let mut args = std::env::args().skip(1);
    let Some(command) = args.next() else {
        help();
        return Ok(0);
    };
    if matches!(command.as_str(), "help" | "--help" | "-h") {
        help();
        return Ok(0);
    }
    if matches!(command.as_str(), "--version" | "version") {
        say!("Phage {}", env!("CARGO_PKG_VERSION"));
        return Ok(0);
    }
    if command != "check" {
        return Err("use phage check FILE.rs".into());
    }
    let file = PathBuf::from(args.next().ok_or("a Rust or LLVM file is required")?);
    let mut function = None;
    let mut unwind = 32usize;
    let mut states = 10000usize;
    let mut timeout = 15000u64;
    let mut memory_mib = budget::default_mib();
    let mut backend = "llvm".to_owned();
    let mut call_depth = 32usize;
    let mut max_slice_len = None;
    while let Some(flag) = args.next() {
        let value = args.next().ok_or_else(|| format!("{flag} needs a value"))?;
        match flag.as_str() {
            "--maxSliceLen" => {
                max_slice_len = Some(value.parse::<u64>().map_err(|_| "invalid slice bound")?)
            }
            "--maxMemoryMiB" => memory_mib = value.parse().map_err(|_| "invalid memory budget")?,
            "--function" => function = Some(value),
            "--engine" => backend = value,
            "--callDepth" => call_depth = value.parse().map_err(|_| "invalid call depth")?,
            "--unwind" | "--blockVisits" => {
                unwind = value.parse().map_err(|_| "invalid unwind limit")?
            }
            "--states" | "--maxStates" => {
                states = value.parse().map_err(|_| "invalid state limit")?
            }
            "--timeout" | "--queryTimeout" => {
                timeout = value
                    .parse()
                    .map_err(|_| "invalid timeout in milliseconds")?
            }
            _ => return Err(format!("unknown option {flag}")),
        }
    }
    let function = function.unwrap_or_else(|| default_entry(&file));
    if !matches!(backend.as_str(), "llvm" | "mir") {
        return Err("--engine must be mir or llvm".into());
    }
    if unwind == 0 || states == 0 || timeout == 0 || call_depth == 0 || memory_mib == 0 {
        return Err("limits must be positive".into());
    }
    if timeout > 86_400_000 {
        return Err("--queryTimeout must be at most one day".into());
    }
    let started = Instant::now();
    let mut run = report::Run::new(&file)?;
    let llvm = run.directory.join(if backend == "mir" {
        "artifact.mir"
    } else {
        "artifact.ll"
    });
    let flags = [
        "--edition=2024",
        "--crate-type=lib",
        "-C",
        "opt-level=1",
        "-C",
        "overflow-checks=yes",
        "-C",
        "panic=abort",
        "-C",
        "codegen-units=1",
        "-C",
        "debuginfo=line-tables-only",
        "-C",
        "llvm-args=-vectorize-loops=false",
        "-C",
        "llvm-args=-vectorize-slp=false",
        "-C",
        "llvm-args=-unroll-threshold=0",
    ];
    let mut labels = Vec::new();
    let mut assumptions = Vec::new();
    let mut locations = std::collections::BTreeMap::new();
    let mut stats = report::SolverStats::default();
    let analysis = (|| -> Result<engine::Verdict, String> {
        budget::Budget::new(memory_mib).check()?;
        if backend == "mir" {
            if file.extension().is_none_or(|e| e != "rs") {
                return Err("MIR frontend requires a Rust source file".into());
            }
            let (bundle, expanded) = mir::frontend::compile(&mut run)?;
            let program = mir::parse::Program::new(&bundle, &expanded)?;
            locations = program.locations.clone();
            let entry = program.entry(&function)?.clone();
            let mut verifier = mir::Verifier::new(program, timeout, unwind, states, call_depth)?;
            verifier.core.solver.budget = budget::Budget::new(memory_mib);
            let verification = verifier.check(&function);
            labels = verifier.core.input_labels.clone();
            assumptions = verifier.assumptions.iter().cloned().collect();
            let solver = &verifier.core.solver;
            stats = report::SolverStats {
                checks: solver.checks,
                assertions: solver.assertions,
                interrupts: solver.interrupts,
                restarts: solver.restarts,
                seconds: solver.elapsed.as_secs_f64(),
                slowest_seconds: solver.slowest_seconds,
                slowest_context: solver.slowest_context.clone(),
                slowest_source: diagnostics::explain(
                    "profile",
                    &solver.slowest_context,
                    &locations,
                )
                .first()
                .and_then(|d| d.source.clone()),
            };
            if !solver.slowest_query.is_empty() {
                std::fs::write(
                    run.directory.join("slowest-query.smt2"),
                    &solver.slowest_query,
                )
                .map_err(|e| e.to_string())?;
            }
            let result = verification?;
            if !solver.counterexample.is_empty() {
                std::fs::write(
                    run.directory.join("counterexample.smt2"),
                    &solver.counterexample,
                )
                .map_err(|e| e.to_string())?;
                let signature = ir::Function {
                    name: entry.name,
                    module: "user".into(),
                    arguments: Vec::new(),
                    parameter_attributes: Vec::new(),
                    params: entry
                        .params
                        .iter()
                        .map(|(n, t)| {
                            Ok((
                                n.clone(),
                                mir::integer(t).ok_or("MIR replay argument unsupported")?.0,
                            ))
                        })
                        .collect::<Result<Vec<_>, String>>()?,
                    blocks: Default::default(),
                    entry: String::new(),
                    locations: Default::default(),
                    noreturn: Default::default(),
                    result_attributes: String::new(),
                };
                run.replay(&signature, &result.model)?;
            }
            return Ok(result);
        }
        if file.extension().is_some_and(|e| e == "ll") {
            std::fs::copy(&file, &llvm).map_err(|e| e.to_string())?;
        } else {
            let deps = run.directory.join("artifact.d");
            // rustc writes codegen-unit temporaries to the output directory
            // (default: cwd, shared by concurrent checks of the same crate).
            let output = Command::new(&run.compiler.compiler)
                .args(flags)
                .arg("--out-dir")
                .arg(&run.directory)
                .arg(format!(
                    "--emit=llvm-ir={},dep-info={}",
                    llvm.display(),
                    deps.display()
                ))
                .arg(&run.source)
                .output()
                .map_err(|e| e.to_string())?;
            std::fs::write(run.directory.join("compiler.stderr"), &output.stderr)
                .map_err(|e| e.to_string())?;
            if !output.status.success() {
                return Err(format!(
                    "rustc failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
            run.dependencies(&deps)?;
        }
        let text = std::fs::read_to_string(&llvm).map_err(|e| e.to_string())?;
        if !text.contains("target triple = \"x86_64") {
            return Err("prototype only supports x86_64 LLVM artifacts".into());
        }
        layout::check(&text)?;
        if text.contains("addrspace(") {
            return Err("non-default address spaces are unsupported".into());
        }
        let module = ir::Module::new("user", text, true)?;
        let target = module.function(&function)?;
        locations = target.locations.clone();
        let mut engine = engine::Engine::new(timeout, unwind, states)?;
        engine.solver.budget = budget::Budget::new(memory_mib);
        engine.call_depth = call_depth;
        engine.max_slice_len = max_slice_len;
        engine.library = engine::Library::Pending(Box::new(run.compiler.clone()));
        engine.modules.push(std::rc::Rc::new(module));
        let verification = engine.verify(&target);
        labels = engine.input_labels;
        assumptions = engine.assumptions.iter().cloned().collect();
        if engine.solver.escalations_decided > 0 {
            assumptions.push(format!(
                "{} solver queries were decided by a racing fresh Z3 or cvc5 (int-blasting)",
                engine.solver.escalations_decided
            ));
        }
        stats = report::SolverStats {
            checks: engine.solver.checks,
            assertions: engine.solver.assertions,
            interrupts: engine.solver.interrupts,
            restarts: engine.solver.restarts,
            seconds: engine.solver.elapsed.as_secs_f64(),
            slowest_seconds: engine.solver.slowest_seconds,
            slowest_context: engine.solver.slowest_context.clone(),
            slowest_source: diagnostics::explain(
                "profile",
                &engine.solver.slowest_context,
                &locations,
            )
            .first()
            .and_then(|d| d.source.clone()),
        };
        if !engine.solver.slowest_query.is_empty() {
            std::fs::write(
                run.directory.join("slowest-query.smt2"),
                &engine.solver.slowest_query,
            )
            .map_err(|e| e.to_string())?;
        }
        let result = verification?;
        if !engine.solver.counterexample.is_empty() {
            std::fs::write(
                run.directory.join("counterexample.smt2"),
                &engine.solver.counterexample,
            )
            .map_err(|e| e.to_string())?;
            run.replay(&target, &result.model)?;
        }
        Ok(result)
    })();
    let mut result = analysis.unwrap_or_else(|error| engine::Verdict {
        status: "unknown".into(),
        detail: error,
        model: String::new(),
        states: 0,
        queries: 0,
        completed: 0,
    });
    if backend == "llvm" && result.status == "unknown" {
        result.detail = diagnostics::llvm_drift(
            &result.detail,
            file.extension()
                .is_some_and(|e| e == "rs")
                .then_some(run.rustc.as_str()),
        );
    }
    let compiler_flags = if backend == "mir" {
        mir::frontend::FLAGS
    } else if file.extension().is_some_and(|e| e == "ll") {
        &[][..]
    } else {
        &flags[..]
    };
    let diagnostics = diagnostics::explain(&result.status, &result.detail, &locations);
    run.save(
        &result,
        &report::Details {
            function: &function,
            flags: compiler_flags,
            limits: (unwind, states, timeout),
            memory_mib,
            max_slice_len,
            elapsed: started.elapsed().as_secs_f64(),
            labels: &labels,
            diagnostics: &diagnostics,
            solver: &stats,
            backend: &backend,
            call_depth,
            assumptions: &assumptions,
        },
    )?;

    say!("{}: {}", result.status, result.detail);
    say!(
        "function: {} | completed paths: {} | states: {} | solver queries: {} | elapsed: {:.3}s",
        function,
        result.completed,
        result.states,
        result.queries,
        started.elapsed().as_secs_f64()
    );
    for assumption in &assumptions {
        say!("assumption: {assumption}");
    }
    for diagnostic in &diagnostics {
        if let Some(source) = &diagnostic.source {
            say!(
                "[{}] {}:{}:{}",
                diagnostic.code,
                source.file,
                source.line,
                source.column
            );
        } else {
            say!("[{}] {}", diagnostic.code, diagnostic.message);
        }
    }
    say!(
        "slowest query: {:.3}s ({})",
        stats.slowest_seconds,
        stats.slowest_context
    );
    say!(
        "solver: {} checks, {} assertions, {:.3}s, {} interrupted, {} restarts",
        stats.checks,
        stats.assertions,
        stats.seconds,
        stats.interrupts,
        stats.restarts
    );
    say!("memory budget: {memory_mib} MiB (sampled process-tree RSS)");
    say!("artifact: {} | block visit bound: {unwind}", llvm.display());
    say!("report: {}", run.directory.join("RESULT.json").display());
    if !result.model.is_empty() && !labels.is_empty() {
        say!(
            "input mapping: {}",
            labels
                .iter()
                .enumerate()
                .map(|(i, name)| format!("input{i}={name}"))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    if !result.model.is_empty() {
        say!("{}", result.model);
    }
    Ok(match result.status.as_str() {
        "proved" => 0,
        "counterexample" => 1,
        _ => 2,
    })
}

fn help() {
    say!(
        "Phage: independent bounded Rust verifier\n\nphage check FILE.rs [--function phage_target] [--blockVisits 32]\n                      [--maxStates 10000] [--queryTimeout 15000]\n                      [--maxMemoryMiB N] (default: min(8192, half available RAM))\n\nThe exported target returns bool; false is a counterexample.\nEntry arguments: integers or bounded reference-shaped byte memory; slices require --maxSliceLen K. Unsupported behavior is unknown.\nZ3 is used as a solver; Kani and CBMC are not invoked."
    );
}

/// `phage_target`, or `ruhealth_target` in files written before the
/// rename (the tool was called RuHealth until 2026-10-01).
fn default_entry(file: &std::path::Path) -> String {
    let text = std::fs::read_to_string(file).unwrap_or_default();
    if !text.contains("phage_target") && text.contains("ruhealth_target") {
        "ruhealth_target".into()
    } else {
        "phage_target".into()
    }
}

/// Environment setting `PHAGE_<name>`, or the pre-rename `RUHEALTH_<name>`.
pub fn setting(name: &str) -> Option<std::ffi::OsString> {
    std::env::var_os(format!("PHAGE_{name}"))
        .or_else(|| std::env::var_os(format!("RUHEALTH_{name}")))
}
