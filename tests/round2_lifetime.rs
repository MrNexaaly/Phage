//! Real CLI spec-conformance controls: start/end distinction, deferred
//! poison, stack coloring, ignore bundles and byte-intrinsic dead reads.
use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
#[test]
fn lifetime_spec_controls() {
    let cases = [
        ("end-only", 0),
        ("poison-transform-unused", 0),
        ("poison-transform-observed", 1),
        ("disjoint-lifetimes-addresses", 1),
        ("overlapping-lifetimes-addresses", 0),
        ("dead-memcpy", 1),
        ("dead-memmove", 1),
        ("zero-dead-memcpy", 0),
        ("zero-dead-memmove", 0),
    ];
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let work = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("phage-round2-{}-{stamp}", std::process::id()));
    fs::create_dir(&work).unwrap();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/round2-lifetime");
    let mut checks = Vec::new();
    for (name, expected) in cases {
        for version in ["old", "new"] {
            checks.push((format!("{name}-{version}.ll"), expected));
        }
    }
    checks.extend([
        ("pure-poison-pointer-transforms.ll".into(), 0),
        ("inttoptr-wide-truncation.ll".into(), 0),
        ("ignore-poison.ll".into(), 0),
        ("ignore-nonnull-failure.ll".into(), 1),
    ]);
    for (name, expected) in checks {
        let output = Command::new(env!("CARGO_BIN_EXE_phage"))
            .current_dir(&work)
            .arg("check")
            .arg(root.join(&name))
            .output()
            .unwrap();
        let text = String::from_utf8_lossy(&output.stdout);
        assert_eq!(
            output.status.code(),
            Some(expected),
            "{name}: {text}\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if name.starts_with("dead-mem") {
            assert!(text.contains("byte-intrinsic read of a stack object outside its lifetime: LangRef's load exception covers load instructions only"),"{text}");
        }
    }
    fs::remove_dir_all(work).unwrap();
}
