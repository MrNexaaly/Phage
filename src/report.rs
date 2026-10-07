//! Immutable run directories, tool/source fingerprints and native replay
//! templates. The LLVM artifact is the authority for the reported verdict.

use crate::{
    diagnostics::{self, Diagnostic},
    engine::Verdict,
    ir::Function,
};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Default)]
pub struct SolverStats {
    pub checks: usize,
    pub assertions: usize,
    /// Cancelled checks resumed in place, and full solver restarts.
    pub interrupts: usize,
    pub restarts: usize,
    pub seconds: f64,
    pub slowest_seconds: f64,
    pub slowest_context: String,
    pub slowest_source: Option<crate::debug::Location>,
}
pub struct Details<'a> {
    pub function: &'a str,
    pub flags: &'a [&'a str],
    pub limits: (usize, usize, u64),
    pub elapsed: f64,
    pub memory_mib: u64,
    pub max_slice_len: Option<u64>,
    pub labels: &'a [String],
    pub diagnostics: &'a [Diagnostic],
    pub solver: &'a SolverStats,
    pub backend: &'a str,
    pub call_depth: usize,
    pub assumptions: &'a [String],
}
pub struct Run {
    pub directory: PathBuf,
    pub source: PathBuf,
    pub rustc: String,
    pub compiler: crate::toolchain::Toolchain,
    pub z3: String,
    pub fingerprints: Vec<(String, String)>,
}

pub fn quote(text: &str) -> String {
    let mut result = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c.is_control() => result.push_str(&format!("\\u{:04x}", u32::from(c))),
            c => result.push(c),
        }
    }
    result.push('"');
    result
}

fn version(tool: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(tool)
        .args(args)
        .output()
        .map_err(|e| format!("{tool}: {e}"))?;
    if !output.status.success() {
        return Err(format!("{tool} version command failed"));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().into())
}

pub fn sha256(path: &Path) -> Result<String, String> {
    let output = Command::new("sha256sum")
        .arg("--")
        .arg(path)
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!("cannot hash {}", path.display()));
    }
    let hash = String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .next()
        .ok_or("hash missing")?
        .to_owned();
    if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("invalid SHA-256 output".into());
    }
    Ok(hash)
}

impl Run {
    pub fn new(source: &Path) -> Result<Self, String> {
        let source = source.canonicalize().map_err(|e| e.to_string())?;
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos();
        let directory = PathBuf::from("results").join(format!("{stamp}-{}", std::process::id()));
        fs::create_dir_all("results").map_err(|e| e.to_string())?;
        fs::create_dir(&directory).map_err(|e| e.to_string())?;
        let mut run = Self {
            directory,
            source,
            rustc: String::new(),
            compiler: crate::toolchain::Toolchain::active()?,
            z3: String::new(),
            fingerprints: Vec::new(),
        };
        // Make the location visible even if an external tool fails to start.
        say!("run: {}", run.directory.display());
        run.capture(&run.source.clone())?;
        run.capture(&std::env::current_exe().map_err(|e| e.to_string())?)?;
        run.rustc = run.compiler.version.trim().into();
        run.capture(&run.compiler.compiler.clone())?;
        run.z3 = version("z3", &["--version"])?;
        Ok(run)
    }
    pub fn capture(&mut self, path: &Path) -> Result<(), String> {
        let path = path.canonicalize().map_err(|e| e.to_string())?;
        let label = path.to_string_lossy().into_owned();
        let hash = sha256(&path)?;
        if let Some((_, before)) = self.fingerprints.iter().find(|(p, _)| *p == label) {
            if *before != hash {
                return Err(format!("source changed during compilation: {label}"));
            }
            return Ok(());
        }
        let snapshot = self
            .directory
            .join("sources")
            .join(path.strip_prefix("/").map_err(|e| e.to_string())?);
        fs::create_dir_all(snapshot.parent().ok_or("snapshot parent missing")?)
            .map_err(|e| e.to_string())?;
        fs::copy(&path, snapshot).map_err(|e| e.to_string())?;
        self.fingerprints.push((label, hash));
        Ok(())
    }
    pub fn dependencies(&mut self, file: &Path) -> Result<(), String> {
        let text = fs::read_to_string(file).map_err(|e| e.to_string())?;
        let deps = text
            .lines()
            .next()
            .and_then(|l| l.split_once(": "))
            .ok_or("invalid compiler dependency file")?
            .1;
        // rustc's makefile dep-info escapes spaces and backslashes.
        let mut paths = Vec::new();
        let mut word = String::new();
        let mut escaped = false;
        for c in deps.chars() {
            if escaped {
                word.push(c);
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c.is_whitespace() {
                if !word.is_empty() {
                    paths.push(std::mem::take(&mut word));
                }
            } else {
                word.push(c);
            }
        }
        if escaped {
            return Err("unfinished dependency path escape".into());
        }
        if !word.is_empty() {
            paths.push(word);
        }
        for path in paths {
            self.capture(Path::new(&path))?;
        }
        Ok(())
    }
    pub fn save(&self, result: &Verdict, details: &Details<'_>) -> Result<(), String> {
        let (unwind, states, timeout) = details.limits;
        let elapsed = details.elapsed;
        let memory_mib = details.memory_mib;
        let slice_bound = details
            .max_slice_len
            .map_or("null".into(), |n| n.to_string());
        let fingerprints = self
            .fingerprints
            .iter()
            .map(|(p, h)| format!("{{\"path\":{},\"sha256\":{}}}", quote(p), quote(h)))
            .collect::<Vec<_>>()
            .join(",");
        let flags = details
            .flags
            .iter()
            .map(|s| quote(s))
            .collect::<Vec<_>>()
            .join(",");
        let labels = details
            .labels
            .iter()
            .map(|s| quote(s))
            .collect::<Vec<_>>()
            .join(",");
        let llvm = self.directory.join(if details.backend == "mir" {
            "artifact.mir"
        } else {
            "artifact.ll"
        });
        let hash = if llvm.is_file() {
            quote(&sha256(&llvm)?)
        } else {
            "null".into()
        };
        let diagnostics = details
            .diagnostics
            .iter()
            .map(diagnostics::json)
            .collect::<Vec<_>>()
            .join(",");
        let assumptions = details
            .assumptions
            .iter()
            .map(|s| quote(s))
            .collect::<Vec<_>>()
            .join(",");
        let solver = details.solver;
        let slowest_source = solver.slowest_source.as_ref().map_or("null".into(), |s| {
            format!(
                "{{\"file\":{},\"line\":{},\"column\":{}}}",
                quote(&s.file),
                s.line,
                s.column
            )
        });
        let interrupts = solver.interrupts;
        let restarts = solver.restarts;
        let legacy_hash = if details.backend == "llvm" {
            hash.clone()
        } else {
            "null".into()
        };
        let json = format!(
            "{{\n\"schema\":4,\"engine\":{},\"callDepth\":{},\"phage\":{},\"status\":{},\"detail\":{},\n\"source\":{},\"function\":{},\"rustc\":{},\"compilerPath\":{},\"compilerSysroot\":{},\"z3\":{},\"compilerFlags\":[{flags}],\n\"artifactSha256\":{hash},\"llvmSha256\":{legacy_hash},\"fingerprints\":[{fingerprints}],\n\"blockVisitBound\":{unwind},\"stateLimit\":{states},\"queryTimeoutMs\":{timeout},\"maxMemoryMiB\":{memory_mib},\"maxSliceLen\":{slice_bound},\n\"completedPaths\":{},\"states\":{},\"queries\":{},\"elapsedSeconds\":{elapsed:.6},\n\"solverChecks\":{},\"solverInterrupts\":{interrupts},\"solverRestarts\":{restarts},\"solverAssertions\":{},\"solverWallSeconds\":{:.6},\"diagnostics\":[{diagnostics}],\n\"slowestQuerySeconds\":{:.6},\"slowestQueryContext\":{},\"slowestQuerySource\":{slowest_source},\n\"inputLabels\":[{labels}],\"model\":{},\"nativeReplay\":\"not run by verifier\",\n\"assumptions\":[{assumptions}],\"scope\":{}\n}}\n",
            quote(details.backend),
            details.call_depth,
            quote(env!("CARGO_PKG_VERSION")),
            quote(&result.status),
            quote(&result.detail),
            quote(&self.source.to_string_lossy()),
            quote(details.function),
            quote(&self.rustc),
            quote(&self.compiler.compiler.to_string_lossy()),
            quote(&self.compiler.sysroot.to_string_lossy()),
            quote(&self.z3),
            result.completed,
            result.states,
            result.queries,
            solver.checks,
            solver.assertions,
            solver.seconds,
            solver.slowest_seconds,
            quote(&solver.slowest_context),
            quote(&result.model),
            quote(if details.backend == "mir" {
                "pinned pre-optimization built-MIR subset; explicit call/block/state bounds; unsupported behavior is unknown"
            } else {
                "compiled sequential LLVM subset; reachable limits and unsupported behavior are unknown"
            })
        );
        fs::write(self.directory.join("RESULT.json"), json).map_err(|e| e.to_string())
    }
    pub fn replay(&self, function: &Function, model: &str) -> Result<(), String> {
        if function.arguments.len() != function.params.len()
            || self.source.extension().is_none_or(|e| e != "rs")
            || !function
                .name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            return Ok(());
        }
        let mut values = Vec::new();
        for i in 0..function.params.len() {
            let marker = format!("(input{i} ");
            let Some((_, tail)) = model.split_once(&marker) else {
                return Ok(());
            };
            let token = tail.split(')').next().ok_or("model token missing")?.trim();
            let value = if let Some(hex) = token.strip_prefix("#x") {
                u128::from_str_radix(hex, 16).ok()
            } else if let Some(binary) = token.strip_prefix("#b") {
                u128::from_str_radix(binary, 2).ok()
            } else {
                None
            };
            let Some(value) = value else { return Ok(()) };
            if function.params[i].1 == 1 {
                return Ok(());
            } // bool cannot be inferred from LLVM i1.
            values.push(format!("{value}u128 as _"));
        }
        let source = format!(
            "//! Native replay from the captured source snapshot. See RESULT.json hashes.\n#![allow(dead_code, unused_imports)]\n#[path = {}]\nmod property;\nfn main() {{\n    let result = std::panic::catch_unwind(|| property::{}({}));\n    match result {{\n        Ok(false) => println!(\"confirmed: property returned false\"),\n        Err(_) => println!(\"confirmed: Rust panicked\"),\n        Ok(true) => {{ eprintln!(\"input did not reproduce a source counterexample\"); std::process::exit(2); }}\n    }}\n}}\n",
            quote(&format!(
                "sources/{}",
                self.source
                    .strip_prefix("/")
                    .map_err(|e| e.to_string())?
                    .display()
            )),
            function.name,
            values.join(", ")
        );
        fs::write(self.directory.join("replay.rs"), source).map_err(|e| e.to_string())
    }
}
