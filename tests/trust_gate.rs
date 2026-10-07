//! Real CLI/Z3 regression controls for artifact semantics and success exits.
use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
const HEADER: &str = "target datalayout = \"e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128\"\ntarget triple = \"x86_64-unknown-linux-gnu\"\n";
struct Sandbox(PathBuf);
impl Sandbox {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("phage-trust-{}-{stamp}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn check(&self, body: &str) -> (i32, String, String) {
        fs::write(self.0.join("property.ll"), format!("{HEADER}{body}")).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_phage"))
            .current_dir(&self.0)
            .args(["check", "property.ll"])
            .output()
            .unwrap();
        let stdout = String::from_utf8(output.stdout).unwrap();
        let report = stdout
            .lines()
            .find_map(|l| l.strip_prefix("report: "))
            .expect("saved report required");
        let json = fs::read_to_string(self.0.join(report)).unwrap();
        (output.status.code().unwrap(), stdout, json)
    }
}
impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn population_count(argument: &str) -> String {
    format!(
        "declare i8 @llvm.ctpop.i8(i8)\ndefine i1 @phage_target() {{\nstart:\n %x = call i8 @llvm.ctpop.i8(i8 {argument})\n %ok = icmp eq i8 %x, 0\n ret i1 %ok\n}}\n"
    )
}
#[test]
fn violated_intrinsic_contract_is_unknown_not_success() {
    let sandbox = Sandbox::new();
    let (code, stdout, json) = sandbox.check(&population_count("range(i8 1, 2) 0"));
    assert_eq!(code, 2, "{stdout}");
    assert!(json.contains("\"status\":\"unknown\""), "{json}");
    assert!(json.contains("\"code\":\"attributePoison\""), "{json}");
    assert!(json.contains("llvm.ctpop.i8"));
    assert!(!json.contains("unannotated call"));
    assert!(json.contains("\"assumptions\":[]"));
    assert!(stdout.contains("LLVM line"));
    let artifact = stdout
        .lines()
        .find_map(|l| l.strip_prefix("artifact: "))
        .unwrap()
        .split(" | ")
        .next()
        .unwrap();
    assert!(
        fs::read_to_string(sandbox.0.join(artifact))
            .unwrap()
            .contains("range(i8 1, 2) 0")
    );
}
#[test]
fn valid_contract_proves_and_noundef_violation_fails() {
    let sandbox = Sandbox::new();
    let (code, _, json) = sandbox.check(&population_count("range(i8 0, 2) 0"));
    assert_eq!(code, 0, "{json}");
    assert!(json.contains("\"status\":\"proved\""));
    let (code, _, json) = sandbox.check(&population_count("noundef range(i8 1, 2) 0"));
    assert_eq!(code, 1, "{json}");
    assert!(json.contains("\"status\":\"counterexample\""));
    assert!(json.contains("\"code\":\"poisonUse\""));
}
#[test]
fn unsupported_and_malformed_artifacts_never_succeed() {
    let sandbox = Sandbox::new();
    for body in [
        "declare i1 @external()\ndefine i1 @phage_target() {\nstart:\n %x = call i1 @external()\n ret i1 %x\n}\n",
        "define i1 @phage_target() {\nstart:\n %x = freeze i1 poison\n ret i1 %x\n}\n",
        "define i1 @phage_target() {\nstart:\n this is not an instruction\n ret i1 true\n}\n",
    ] {
        let (code, _, json) = sandbox.check(body);
        assert_eq!(code, 2, "{json}");
        assert!(json.contains("\"status\":\"unknown\""));
    }
}
