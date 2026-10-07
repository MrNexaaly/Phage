//! Tool ordering and mismatched/missing version negative controls. Fake
//! executables exercise real --version commands without changing global env.
use super::*;
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    time::{SystemTime, UNIX_EPOCH},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("phage-tools-{}-{stamp}", std::process::id()));
        fs::create_dir_all(root.join("sysroot")).unwrap();
        fs::create_dir_all(root.join("path")).unwrap();
        Self(root)
    }
    fn compiler(&self) -> Toolchain {
        Toolchain {
            compiler: self.0.join("bin/rustc"),
            sysroot: self.0.clone(),
            version: "release: 1.99.0\nLLVM version: 23.1.1\n".into(),
            commit: "test".into(),
            llvm: "23.1.1".into(),
            libraries: self.0.join("lib"),
            bin: self.0.join("sysroot"),
            name: "stable".into(),
        }
    }
    fn tool(&self, directory: &str, name: &str, version: &str) -> PathBuf {
        let path = self.0.join(directory).join(name);
        fs::write(&path, format!("#!/bin/sh\nprintf '%s\\n' '{version}'\n")).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        path
    }
    fn paths(&self) -> Vec<PathBuf> {
        vec![self.0.join("path")]
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn sysroot_precedes_versioned_path_and_bare_tool() {
    let f = Fixture::new();
    let sysroot = f.tool("sysroot", "llvm-link", "LLVM version 23.1.1");
    let versioned = f.tool("path", "llvm-link-23", "LLVM version 23.0.0");
    let bare = f.tool("path", "llvm-link", "LLVM version 23.1.1");
    assert_eq!(
        f.compiler().tool_in("llvm-link", &f.paths()).unwrap(),
        sysroot
    );
    fs::remove_file(sysroot).unwrap();
    assert_eq!(
        f.compiler().tool_in("llvm-link", &f.paths()).unwrap(),
        versioned
    );
    fs::remove_file(versioned).unwrap();
    assert_eq!(f.compiler().tool_in("llvm-link", &f.paths()).unwrap(), bare);
}

#[test]
fn mismatch_reports_both_versions_and_exact_component_fix() {
    let f = Fixture::new();
    f.tool("path", "llvm-link", "LLVM version 22.1.8");
    let error = f.compiler().tool_in("llvm-link", &f.paths()).unwrap_err();
    for required in [
        "LLVM 22.1.8",
        "LLVM 23.1.1",
        "rustc 1.99.0",
        "rustup component add llvm-tools --toolchain stable",
    ] {
        assert!(error.contains(required), "{error}");
    }
}

#[test]
fn wrong_or_unreadable_version_never_becomes_a_matching_tool() {
    let f = Fixture::new();
    f.tool("sysroot", "llvm-dis", "GNU version 23.1.1");
    f.tool("path", "llvm-dis-23", "LLVM version 22.1.8");
    assert!(f.compiler().tool_in("llvm-dis", &f.paths()).is_err());
    let bare = f.tool("path", "llvm-dis", "LLVM version 23.1.1");
    assert_eq!(f.compiler().tool_in("llvm-dis", &f.paths()).unwrap(), bare);
}

#[test]
fn missing_tool_is_an_actionable_error() {
    let f = Fixture::new();
    let error = f.compiler().tool_in("llvm-dis", &f.paths()).unwrap_err();
    assert!(error.contains("tool missing"));
    assert!(error.contains("--toolchain stable"));
}

#[test]
fn metadata_comes_from_the_concrete_sysroot_compiler() {
    let f = Fixture::new();
    let root = f.0.join("concrete");
    fs::create_dir_all(root.join("bin")).unwrap();
    let compiler = root.join("bin/rustc");
    fs::write(&compiler,format!("#!/bin/sh\nif [ \"$1\" = '-vV' ]; then\nprintf '%s\\n' 'release: 1.99.0' 'commit-hash: immutable' 'host: x86_64-unknown-linux-gnu' 'LLVM version: 23.1.1'\nelse\nprintf '%s\\n' '{}'\nfi\n",root.display())).unwrap();
    fs::set_permissions(&compiler, fs::Permissions::from_mode(0o755)).unwrap();
    let resolved = Toolchain::resolve(&root, Some("stable".into())).unwrap();
    assert_eq!(resolved.compiler, compiler);
    assert_eq!(resolved.sysroot, root);
    assert_eq!(resolved.commit, "immutable");
    assert!(resolved.version.contains("23.1.1"));
}

#[test]
fn busy_tool_is_retried_not_skipped() {
    let f = Fixture::new();
    let path = f.tool("sysroot", "llvm-link", "LLVM version 23.1.1");
    // An open write handle makes exec fail with ETXTBSY until it is closed.
    let writer = fs::OpenOptions::new().append(true).open(&path).unwrap();
    let release = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(20));
        drop(writer);
    });
    assert_eq!(f.compiler().tool_in("llvm-link", &f.paths()).unwrap(), path);
    release.join().unwrap();
}
