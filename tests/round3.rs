//! Spec negative controls and real CLI memory-budget exhaustion.
use std::{fs, path::PathBuf, process::Command};
#[test]
fn round3_spec_and_budget_controls() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/round3");
    let work =
        PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("round3-{}", std::process::id()));
    fs::create_dir_all(&work).unwrap();
    let mut cases: Vec<_> = [
        "fabricated-heap",
        "fabricated-native",
        "fabricated-after",
        "fabricated-stack",
        "fabricated-access",
    ]
    .into_iter()
    .map(|n| (n.to_owned(), 1))
    .collect();
    for kind in ["memcpy", "memmove", "memset"] {
        for (case, exit) in [
            ("bad", 1),
            ("aligned", 0),
            ("plain", 0),
            ("poison-align", 2),
        ] {
            cases.push((format!("zero-{kind}-{case}"), exit));
        }
    }
    for version in ["old", "new"] {
        cases.push((format!("live-start-{version}-reset"), 1));
        cases.push((format!("live-start-{version}-restore"), 0));
    }
    for (case, exit) in cases {
        let output = Command::new(env!("CARGO_BIN_EXE_phage"))
            .current_dir(&work)
            .arg("check")
            .arg(root.join(format!("{case}.ll")))
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(exit),
            "{case}: {} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let output = Command::new(env!("CARGO_BIN_EXE_phage"))
        .current_dir(&work)
        .arg("check")
        .arg(root.join("memory-growth.ll"))
        .args([
            "--maxMemoryMiB",
            "64",
            "--maxStates",
            "10000",
            "--blockVisits",
            "10000",
        ])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(2), "{text}");
    assert!(text.contains("memory budget exceeded (64 MiB)"), "{text}");
    let run = text.lines().find_map(|l| l.strip_prefix("run: ")).unwrap();
    let report = fs::read_to_string(work.join(run).join("RESULT.json")).unwrap();
    assert!(report.contains("\"maxMemoryMiB\":64"), "{report}");
    assert!(report.contains("memoryLimit"), "{report}");
    fs::remove_dir_all(work).unwrap();
}
