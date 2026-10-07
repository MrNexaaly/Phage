//! Active rustc metadata and matching LLVM tool discovery. All subprocesses
//! use the caller's rustup selection (environment and working directory).
//! Never feed bitcode to an unchecked PATH tool from another LLVM major.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Clone)]
pub struct Toolchain {
    pub compiler: PathBuf,
    pub sysroot: PathBuf,
    pub version: String,
    pub commit: String,
    pub llvm: String,
    pub libraries: PathBuf,
    bin: PathBuf,
    name: String,
}

impl Toolchain {
    pub fn active() -> Result<Self, String> {
        let selected = output(Path::new("rustc"), &["--print", "sysroot"])?;
        Self::resolve(
            Path::new(selected.trim()),
            std::env::var("RUSTUP_TOOLCHAIN").ok(),
        )
    }

    pub fn named(name: &str) -> Result<Self, String> {
        let output = Command::new("rustup")
            .args(["run", name, "rustc", "--print", "sysroot"])
            .output()
            .map_err(|e| e.to_string())?;
        if !output.status.success() {
            return Err(format!("compiler toolchain {name} is unavailable"));
        }
        Self::resolve(
            Path::new(String::from_utf8_lossy(&output.stdout).trim()),
            Some(name.into()),
        )
    }

    fn resolve(selected: &Path, name: Option<String>) -> Result<Self, String> {
        let compiler = std::fs::canonicalize(selected.join("bin/rustc"))
            .map_err(|e| format!("resolve rustc in {}: {e}", selected.display()))?;
        let version = output(&compiler, &["-vV"])?;
        let sysroot = PathBuf::from(output(&compiler, &["--print", "sysroot"])?.trim());
        let commit = field(&version, "commit-hash:")?;
        let llvm = field(&version, "LLVM version:")?;
        let host = field(&version, "host:")?;
        let name = name
            .or_else(|| {
                sysroot
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
            })
            .unwrap_or(field(&version, "release:")?);
        let root = sysroot.join("lib/rustlib").join(host);
        Ok(Self {
            compiler,
            sysroot,
            version,
            commit,
            llvm,
            libraries: root.join("lib"),
            bin: root.join("bin"),
            name,
        })
    }

    pub fn tool(&self, name: &str) -> Result<PathBuf, String> {
        let paths: Vec<_> = std::env::var_os("PATH")
            .map(|p| std::env::split_paths(&p).collect())
            .unwrap_or_default();
        self.tool_in(name, &paths)
    }

    fn tool_in(&self, name: &str, paths: &[PathBuf]) -> Result<PathBuf, String> {
        let major = self
            .llvm
            .split('.')
            .next()
            .ok_or("rustc LLVM version missing")?;
        let versioned = format!("{name}-{major}");
        let candidates = std::iter::once(self.bin.join(name))
            .chain(paths.iter().map(|p| p.join(&versioned)))
            .chain(paths.iter().map(|p| p.join(name)));
        let mut rejected = Vec::new();
        for path in candidates.filter(|p| p.is_file()) {
            let version = match output(&path, &["--version"]) {
                Ok(v) => v,
                Err(e) => {
                    rejected.push(e);
                    continue;
                }
            };
            match llvm_version(&version) {
                Some(v) if v.split('.').next() == Some(major) => return Ok(path),
                Some(v) => rejected.push(format!(
                    "{} is LLVM {v}, rustc uses LLVM {}",
                    path.display(),
                    self.llvm
                )),
                None => rejected.push(format!(
                    "{} has no recognized LLVM version: {}",
                    path.display(),
                    version.trim()
                )),
            }
        }
        Err(format!(
            "no matching {name} for rustc {} / LLVM {}: {}. Fix: rustup component add llvm-tools --toolchain {}",
            field(&self.version, "release:")?,
            self.llvm,
            if rejected.is_empty() {
                "tool missing".into()
            } else {
                rejected.join("; ")
            },
            self.name
        ))
    }
}

pub fn field(version: &str, key: &str) -> Result<String, String> {
    version
        .lines()
        .find_map(|l| l.strip_prefix(key))
        .map(|v| v.trim().to_owned())
        .ok_or_else(|| format!("rustc -vV lacks {key}"))
}

fn llvm_version(version: &str) -> Option<&str> {
    version.lines().find_map(|l| {
        let (_, tail) = l.split_once("version ")?;
        let v = tail.split_whitespace().next()?;
        v.split('.').next()?.parse::<u32>().ok()?;
        l.contains("LLVM").then_some(v)
    })
}

fn output(program: &Path, args: &[&str]) -> Result<String, String> {
    // ETXTBSY is transient: another thread's fork can briefly inherit a write fd
    // to a just-written tool. Retrying keeps candidate order deterministic
    // instead of silently falling through to a lower-priority tool.
    let mut delay = std::time::Duration::from_millis(1);
    let result = loop {
        match Command::new(program).args(args).output() {
            Err(e)
                if e.kind() == std::io::ErrorKind::ExecutableFileBusy
                    && delay.as_millis() < 256 =>
            {
                std::thread::sleep(delay);
                delay *= 2;
            }
            other => break other.map_err(|e| format!("{}: {e}", program.display()))?,
        }
    };
    if !result.status.success() {
        return Err(format!(
            "{} {args:?} failed: {}",
            program.display(),
            String::from_utf8_lossy(&result.stderr).trim()
        ));
    }
    String::from_utf8(result.stdout).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests;
