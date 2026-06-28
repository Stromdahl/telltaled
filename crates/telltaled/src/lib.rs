//! `telltaled` — a system telemetry daemon.
//!
//! A "telltale" is an indicator that reveals an otherwise-hidden condition (see
//! `docs/adr/0001-name-the-system-telemetry-daemon-telltaled.md`). This crate is
//! the daemon's testable core: collection logic lives here as pure, unit-testable
//! functions; the binary (`src/main.rs`) is a thin shim over it.
//!
//! Overriding constraint: **low host overhead** — measuring a machine must never
//! meaningfully perturb it (see `AGENTS.md`).
//!
//! The first collector is [`loadavg`] (M0, #1): a pure parser over
//! `/proc/loadavg`. New signals follow the same shape — pure logic here, I/O in
//! the shim.

pub mod config;
pub mod loadavg;

use std::sync::mpsc::Receiver;

/// A sink for emitting telemetry lines. Errors are best-effort: [`run_loop`]
/// logs them to stderr and continues (ADR 0003).
pub trait Sink {
    fn emit(&mut self, line: &str) -> Result<(), Box<dyn std::error::Error>>;
}

/// The interruptible wait between loop iterations.
pub trait Clock {
    /// Wait up to `secs` seconds; return `true` if a shutdown signal arrived
    /// (a `()` received or channel closed), `false` if the timeout elapsed normally.
    fn wait_with_interrupt(&self, secs: u64, rx: &Receiver<()>) -> bool;
}

/// Resident sample-and-emit loop (M1, #7). Reads `path` (e.g. `/proc/loadavg`),
/// emits a `load …` line to `sink`, waits `interval_secs`, then repeats until
/// `shutdown` signals termination.
pub fn run_loop<S: Sink, C: Clock>(
    path: &str,
    sink: &mut S,
    clock: &C,
    interval_secs: u64,
    shutdown: &Receiver<()>,
) {
    loop {
        emit_sample(path, sink);
        if clock.wait_with_interrupt(interval_secs, shutdown) {
            break;
        }
    }
}

fn emit_sample<S: Sink>(path: &str, sink: &mut S) {
    match read_sample(path) {
        Ok(line) => {
            if let Err(e) = sink.emit(&line) {
                eprintln!("telltaled: emit: {e}");
            }
        }
        Err(e) => eprintln!("telltaled: sample: {e}"),
    }
}

fn read_sample(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let contents = std::fs::read_to_string(path).map_err(|e| format!("reading {path}: {e}"))?;
    let sample = loadavg::parse_loadavg(&contents).map_err(|e| format!("parsing: {e}"))?;
    Ok(sample.to_string())
}

/// The running daemon's version, taken from the crate metadata.
#[must_use]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    struct VecSink(Vec<String>);

    impl Sink for VecSink {
        fn emit(&mut self, line: &str) -> Result<(), Box<dyn std::error::Error>> {
            self.0.push(line.to_string());
            Ok(())
        }
    }

    struct FailSink(Vec<String>);

    impl Sink for FailSink {
        fn emit(&mut self, line: &str) -> Result<(), Box<dyn std::error::Error>> {
            self.0.push(line.to_string());
            Err("injected sink failure".into())
        }
    }

    /// FakeClock returns immediately; after `waits` non-interrupting calls it
    /// signals interrupted so the loop terminates.
    struct FakeClock {
        waits_left: std::cell::RefCell<usize>,
    }

    impl FakeClock {
        fn new(waits: usize) -> Self {
            FakeClock {
                waits_left: std::cell::RefCell::new(waits),
            }
        }
    }

    impl Clock for FakeClock {
        fn wait_with_interrupt(&self, _secs: u64, _rx: &mpsc::Receiver<()>) -> bool {
            let mut w = self.waits_left.borrow_mut();
            if *w == 0 {
                return true;
            }
            *w -= 1;
            false
        }
    }

    #[test]
    fn version_is_reported() {
        assert!(!version().is_empty());
    }

    #[test]
    fn run_loop_sample_first_and_correct_count() {
        // 3 emits: FakeClock allows 2 non-interrupting waits before shutdown.
        let (_shutdown_tx, rx) = mpsc::channel::<()>();
        let mut sink = VecSink(Vec::new());
        let clock = FakeClock::new(2);
        run_loop("/proc/loadavg", &mut sink, &clock, 60, &rx);
        assert_eq!(sink.0.len(), 3, "expected 3 emits");
        assert!(
            sink.0[0].starts_with("load "),
            "first emit is a load line (sample-first)"
        );
    }

    #[test]
    fn run_loop_continues_after_sink_failure() {
        // A sink that always errors must not stop iteration (ADR 0003).
        let (_shutdown_tx, rx) = mpsc::channel::<()>();
        let mut sink = FailSink(Vec::new());
        let clock = FakeClock::new(2);
        run_loop("/proc/loadavg", &mut sink, &clock, 60, &rx);
        assert_eq!(
            sink.0.len(),
            3,
            "all iterations attempted despite sink errors"
        );
    }
}
