//! Pinned nightly frontend that captures built MIR before optimization.
//! Compiler failure, wrong host/version and a missing early dump are unknown.

use crate::report::Run;
use std::{fs, path::PathBuf, process::Command};

pub const TOOLCHAIN: &str = "nightly-2026-08-21";
const COMMIT: &str = "8925ea358a0f265ca61026aadc7ecc506c545cbe";
pub const FLAGS: &[&str] = &[
    "--edition=2024",
    "--crate-type=lib",
    "-C",
    "overflow-checks=yes",
    "-C",
    "panic=abort",
    "-Zmir-opt-level=0",
    "-Zmir-preserve-ub=yes",
    "-Zmir-include-spans=yes",
    "-Zdump-mir=built",
];

pub fn compile(run: &mut Run) -> Result<(String, String), String> {
    run.compiler = crate::toolchain::Toolchain::named(TOOLCHAIN)?;
    let version = &run.compiler.version;
    if !version.contains(COMMIT) || !version.contains("host: x86_64-unknown-linux-gnu") {
        return Err("MIR compiler version/host is not supported".into());
    }
    run.rustc = version.trim().into();
    run.capture(&run.compiler.compiler.clone())?;
    let directory = run.directory.join("built-mir");
    fs::create_dir(&directory).map_err(|e| e.to_string())?;
    let dep = run.directory.join("artifact.d");
    let emitted = run.directory.join("emitted.mir");
    let output = Command::new(&run.compiler.compiler)
        .args(FLAGS)
        .arg(format!("-Zdump-mir-dir={}", directory.display()))
        .arg(format!(
            "--emit=mir={},dep-info={}",
            emitted.display(),
            dep.display()
        ))
        .arg(&run.source)
        .output()
        .map_err(|e| e.to_string())?;
    fs::write(run.directory.join("compiler.stderr"), &output.stderr).map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "rustc failed ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    run.dependencies(&dep)?;
    let expanded = Command::new(&run.compiler.compiler)
        .args(["--edition=2024", "--crate-type=lib", "-Zunpretty=expanded"])
        .arg(&run.source)
        .output()
        .map_err(|e| e.to_string())?;
    if !expanded.status.success() {
        return Err(format!(
            "rustc failed to expand declarations: {}",
            String::from_utf8_lossy(&expanded.stderr)
        ));
    }
    let expanded = String::from_utf8(expanded.stdout).map_err(|e| e.to_string())?;
    fs::write(run.directory.join("expanded.rs"), &expanded).map_err(|e| e.to_string())?;
    let mut files = fs::read_dir(&directory)
        .map_err(|e| e.to_string())?
        .map(|e| e.map(|e| e.path()))
        .collect::<Result<Vec<PathBuf>, _>>()
        .map_err(|e| e.to_string())?;
    files.sort();
    let mut bundle =
        String::from("// Phage pinned built-MIR bundle; no optimized-body fallback.\n");
    let mut count = 0;
    for file in files {
        let filename = file
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or("MIR dump name invalid")?;
        if !filename.ends_with(".built.after.mir") {
            continue;
        }
        let text = fs::read_to_string(&file).map_err(|e| e.to_string())?;
        if !text.lines().any(|l| l.starts_with("fn ")) {
            continue;
        }
        let path = filename
            .split('.')
            .nth(1)
            .ok_or("MIR definition path missing")?;
        let key = path.replace('-', "::");
        bundle.push_str(&format!("// phage-function: {key}\n{text}\n"));
        count += 1;
    }
    if count == 0 {
        return Err("compiler produced no supported built-MIR function dumps".into());
    }
    fs::write(run.directory.join("artifact.mir"), &bundle).map_err(|e| e.to_string())?;
    Ok((bundle, expanded))
}
