//! End-to-end Rust ABI coverage. Direct LLVM negative controls live in entry/tests.rs.
use std::{fs, path::PathBuf, process::Command};
#[test]
fn compiled_byte_buffer_controls() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let work = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("byte-buffers-{}", std::process::id()));
    fs::create_dir_all(&work).unwrap();
    for (function, bound, expected) in [
        ("parser", false, 0),
        ("bad_parser", false, 1),
        ("phage_target", false, 0),
        ("mutable", false, 0),
        ("slice", true, 0),
        ("mutable_slice", true, 0),
        ("bad_value", false, 1),
        ("bad_slice", true, 1),
        ("slice", false, 2),
        ("unsupported", false, 2),
    ] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_phage"));
        command
            .current_dir(&work)
            .arg("check")
            .arg(root.join("fixtures/byte-buffers/check.rs"))
            .args(["--function", function]);
        if bound {
            command.args(["--maxSliceLen", "16"]);
        }
        let output = command.output().unwrap();
        assert_eq!(
            output.status.code(),
            Some(expected),
            "{function}: {} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    // Retain run artifacts, including the deliberately failing witnesses.
}
