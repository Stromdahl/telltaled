//! End-to-end check of the `telltaled` binary (M1, #7): the daemon emits a
//! `load …` line immediately on start (sample-first), then runs as a resident
//! loop. This test reads the first line and kills the process — it does not
//! wait for a natural exit, since the loop runs until interrupted (SIGTERM
//! handling is a sibling issue).

use std::io::{BufRead as _, BufReader};
use std::process::{Command, Stdio};

/// `CARGO_BIN_EXE_<name>` is set by Cargo for integration tests of a binary.
const BIN: &str = env!("CARGO_BIN_EXE_telltaled");

#[test]
fn emits_load_sample_immediately_then_stays_resident() {
    let mut child = Command::new(BIN)
        .stdout(Stdio::piped())
        .spawn()
        .expect("the telltaled binary should be runnable");

    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);
    let mut first_line = String::new();
    reader
        .read_line(&mut first_line)
        .expect("should read at least one line on start");

    assert!(
        first_line.trim_end().starts_with("load "),
        "expected a `load …` line on start, got {:?}",
        first_line,
    );

    // Resident loop: process is still running. Kill it and reap.
    child.kill().expect("should be able to kill telltaled");
    child.wait().expect("should be able to wait for telltaled");
}
