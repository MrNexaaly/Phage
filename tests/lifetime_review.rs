//! Real CLI regressions for the independent LLVM 23 soundness review. The
//! original .x.ll repros are retained verbatim; old/new variants stay tested.
use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
#[test]
fn reviewed_lifetimes_and_panic_names_have_the_required_verdicts() {
    let cases = [
        ("prestart.x.ll", 1),
        ("undef-marker.ll", 2),
        ("indirect-name-collision.ll", 0),
        ("prestart_old.x.ll", 1),
        ("deadload.x.ll", 0),
        ("deadload-branch.ll", 1),
        ("heap-revive-old.ll", 2),
        ("heap-revive-sized-old.ll", 2),
        ("heap-revive-new.ll", 2),
        ("panic-declared-unresolved.ll", 2),
        ("panic-defined-legacy.ll", 0),
        ("panic-defined-v0.ll", 0),
        ("poison-direct.ll", 0),
        ("poison-end.ll", 0),
        ("caller-start-old.ll", 0),
        ("caller-start-new.ll", 0),
        ("caller-prestart-old.ll", 1),
        ("caller-prestart-new.ll", 1),
        ("deadload-noundef-old.ll", 1),
        ("deadload-noundef-new.ll", 1),
        ("poison-start-preserves-old.ll", 0),
        ("poison-start-preserves-new.ll", 0),
        ("conditional-poison-old.ll", 2),
        ("conditional-poison-new.ll", 2),
        ("prestart-load-old.ll", 0),
        ("prestart-load-new.ll", 0),
    ];
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let work = std::env::temp_dir().join(format!(
        "phage-lifetime-review-{}-{stamp}",
        std::process::id()
    ));
    fs::create_dir(&work).unwrap();
    for (name, expected) in cases {
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures/lifetime-review")
            .join(name);
        let output = Command::new(env!("CARGO_BIN_EXE_phage"))
            .current_dir(&work)
            .args(["check"])
            .arg(source)
            .output()
            .unwrap();
        let text = String::from_utf8_lossy(&output.stdout);
        assert_eq!(
            output.status.code(),
            Some(expected),
            "{name}: {text}\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let path = text
            .lines()
            .find_map(|l| l.strip_prefix("report: "))
            .expect("report required");
        let report = fs::read_to_string(work.join(path)).unwrap();
        assert!(report.contains("\"compilerPath\":\"/"));
    }
    fs::remove_dir_all(work).unwrap();
}
