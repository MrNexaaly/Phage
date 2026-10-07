//! Real compiled std operations and the frozen IPC decoder: exact symbolic
//! lengths must support positive cases, while off-by-one/capacity overflow fail.
use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
#[test]
fn bounded_std_allocations_and_ipc() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let work = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("phage-symalloc-{}-{stamp}", std::process::id()));
    fs::create_dir(&work).unwrap();
    let cases = [
        ("phage_target", 0),
        ("extended", 0),
        ("copied", 0),
        ("utf8", 0),
        ("off_by_one", 1),
        ("uncapped", 1),
    ];
    for (function, expected) in cases {
        let output = Command::new(env!("CARGO_BIN_EXE_phage"))
            .current_dir(&work)
            .arg("check")
            .arg(root.join("examples/symbolic-alloc.rs"))
            .args([
                "--function",
                function,
                "--blockVisits",
                "96",
                "--maxStates",
                "200000",
            ])
            .output()
            .unwrap();
        let text = String::from_utf8_lossy(&output.stdout);
        assert_eq!(
            output.status.code(),
            Some(expected),
            "{function}: {text}\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let output = Command::new(env!("CARGO_BIN_EXE_phage"))
        .current_dir(&work)
        .arg("check")
        .arg(root.join("fixtures/ipc-decode/check.rs"))
        .args(["--maxStates", "200000"])
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "IPC: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    fs::remove_dir_all(work).unwrap();
}
