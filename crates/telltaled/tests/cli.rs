//! End-to-end check of the `telltaled` binary (M0, issues/001): one invocation
//! reads `/proc/loadavg`, prints a single sample line, and exits 0. This observes
//! the exit code and stdout shape the way an operator would, complementing the
//! pure-parser unit/property tests in the library.

use std::process::Command;

/// `CARGO_BIN_EXE_<name>` is set by Cargo for integration tests of a binary.
const BIN: &str = env!("CARGO_BIN_EXE_telltaled");

#[test]
fn prints_one_load_sample_and_exits_zero() {
    let output = Command::new(BIN)
        .output()
        .expect("the telltaled binary should be runnable");

    assert!(
        output.status.success(),
        "expected exit 0, got {:?}; stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr),
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1, "expected exactly one line, got {stdout:?}");
    assert!(
        lines[0].starts_with("load "),
        "unexpected sample line: {:?}",
        lines[0],
    );
}
