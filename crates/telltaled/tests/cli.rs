//! End-to-end checks of the `telltaled` binary (M1, #7 and #8):
//! - sample-first: the daemon emits a `load …` line immediately on start.
//! - graceful shutdown: SIGTERM and SIGINT each wake the interruptible wait
//!   and the daemon exits 0 promptly, rather than waiting out the full 60 s interval.

use std::io::{BufRead as _, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

/// `CARGO_BIN_EXE_<name>` is set by Cargo for integration tests of a binary.
const BIN: &str = env!("CARGO_BIN_EXE_telltaled");

/// Sends `kill -<sig>` to `child` and asserts it exits 0 within 2 s.
#[allow(clippy::expect_used, clippy::panic)]
fn send_signal_and_assert_clean_exit(mut child: Child, sig: &str) {
    let pid = child.id();
    Command::new("kill")
        .arg(format!("-{sig}"))
        .arg(pid.to_string())
        .status()
        .expect("`kill` command should run");

    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(child.wait());
    });
    match rx.recv_timeout(Duration::from_secs(2)) {
        Ok(Ok(status)) => assert!(
            status.success(),
            "expected exit 0 after SIG{sig}, got {:?}",
            status,
        ),
        Ok(Err(e)) => panic!("child.wait() failed: {e}"),
        Err(_) => panic!("telltaled did not exit within 2 s after SIG{sig}"),
    }
}

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
fn sigterm_causes_clean_prompt_shutdown() {
    let mut child = Command::new(BIN)
        .stdout(Stdio::piped())
        .spawn()
        .expect("the telltaled binary should be runnable");

    // Verify sample-first line arrives before the signal.
    let stdout = child.stdout.take().expect("stdout is piped");
    let mut reader = BufReader::new(stdout);
    let mut first_line = String::new();
    reader
        .read_line(&mut first_line)
        .expect("should read at least one line before SIGTERM");
    assert!(
        first_line.trim_end().starts_with("load "),
        "expected a `load …` line before SIGTERM, got {:?}",
        first_line,
    );

    send_signal_and_assert_clean_exit(child, "TERM");
}

#[test]
fn sigint_causes_clean_prompt_shutdown() {
    let mut child = Command::new(BIN)
        .stdout(Stdio::piped())
        .spawn()
        .expect("the telltaled binary should be runnable");

    // Verify sample-first line arrives before the signal.
    let stdout = child.stdout.take().expect("stdout is piped");
    let mut reader = BufReader::new(stdout);
    let mut first_line = String::new();
    reader
        .read_line(&mut first_line)
        .expect("should read at least one line before SIGINT");
    assert!(
        first_line.trim_end().starts_with("load "),
        "expected a `load …` line before SIGINT, got {:?}",
        first_line,
    );

    send_signal_and_assert_clean_exit(child, "INT");
}
