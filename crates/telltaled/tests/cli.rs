//! End-to-end checks of the `telltaled` binary (M1, #7/#8): the daemon emits
//! a `load …` line immediately on start (sample-first), runs as a resident
//! loop, and shuts down promptly and cleanly on SIGTERM (#8).

use std::io::{BufRead as _, BufReader};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

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

#[test]
fn sigterm_causes_prompt_clean_shutdown() {
    let mut child = Command::new(BIN)
        .stdout(Stdio::piped())
        .spawn()
        .expect("the telltaled binary should be runnable");

    let pid = child.id();

    // Confirm sample-first: first line appears before SIGTERM is sent.
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

    // Send SIGTERM via system kill(1) — avoids unsafe libc/nix.
    Command::new("kill")
        .args(["-TERM", &pid.to_string()])
        .status()
        .expect("kill should succeed");

    // Assert clean (exit 0) shutdown within ~2 s, proving the signal woke the
    // interruptible wait rather than waiting out the full 60 s interval.
    let deadline = Instant::now() + Duration::from_secs(2);
    let status = loop {
        match child.try_wait().expect("try_wait should not fail") {
            Some(s) => break s,
            None if Instant::now() >= deadline => {
                child.kill().ok();
                panic!("process did not exit within 2 s after SIGTERM");
            }
            None => std::thread::sleep(Duration::from_millis(50)),
        }
    };
    assert!(
        status.success(),
        "expected exit 0 after SIGTERM, got {:?}",
        status.code(),
    );
}
