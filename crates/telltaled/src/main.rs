//! `telltaled` binary entry point — a thin shim over the `telltaled` library.
//!
//! M1 (#7): resident sample-and-emit loop. Reads `/proc/loadavg` once per
//! interval (default 60 s), prints `load …` to stdout immediately on start,
//! then once per interval. The wait is interruptible via an mpsc channel so a
//! future sibling can wake it on SIGTERM without polling.

use std::io::Write as _;
use std::process::ExitCode;
use std::sync::mpsc;
use std::time::Duration;
use telltaled::{Clock, Sink};

/// The kernel file exposing the 1/5/15-minute load averages.
const LOADAVG_PATH: &str = "/proc/loadavg";

/// Default sampling interval in seconds.
const INTERVAL_SECS: u64 = 60;

fn main() -> ExitCode {
    let (_shutdown_tx, shutdown_rx) = mpsc::channel::<()>();
    let mut sink = StdoutSink;
    let clock = RealClock;
    telltaled::run_loop(LOADAVG_PATH, &mut sink, &clock, INTERVAL_SECS, &shutdown_rx);
    ExitCode::SUCCESS
}

struct StdoutSink;

impl Sink for StdoutSink {
    fn emit(&mut self, line: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut out = std::io::stdout().lock();
        writeln!(out, "{line}").map_err(Into::into)
    }
}

struct RealClock;

impl Clock for RealClock {
    fn wait_with_interrupt(&self, secs: u64, rx: &mpsc::Receiver<()>) -> bool {
        use std::sync::mpsc::RecvTimeoutError;
        match rx.recv_timeout(Duration::from_secs(secs)) {
            Ok(()) | Err(RecvTimeoutError::Disconnected) => true,
            Err(RecvTimeoutError::Timeout) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn stdout_sink_emit_returns_ok() {
        let mut sink = StdoutSink;
        assert!(sink.emit("load 0.00 0.00 0.00").is_ok());
    }

    #[test]
    fn real_clock_returns_true_when_signal_pending() {
        let (tx, rx) = mpsc::channel::<()>();
        tx.send(()).unwrap();
        assert!(RealClock.wait_with_interrupt(60, &rx));
    }

    #[test]
    fn real_clock_returns_false_on_timeout() {
        let (_tx, rx) = mpsc::channel::<()>();
        assert!(!RealClock.wait_with_interrupt(0, &rx));
    }

    #[test]
    fn real_clock_returns_true_when_channel_closed() {
        let (tx, rx) = mpsc::channel::<()>();
        drop(tx);
        assert!(RealClock.wait_with_interrupt(0, &rx));
    }
}
