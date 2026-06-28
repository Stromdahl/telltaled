//! `telltaled` binary entry point — a thin shim over the `telltaled` library.
//!
//! M1 (#7, #8): resident sample-and-emit loop. Reads `/proc/loadavg` once per
//! interval (default 60 s), prints `load …` to stdout immediately on start,
//! then once per interval. A signal thread (#8) sends on a shutdown channel
//! when SIGTERM or SIGINT arrives, waking the interruptible wait immediately.
//!
//! M2 (#3): interval and push credentials come from environment variables via
//! [`telltaled::config::parse_config`]; missing required vars exit non-zero.

use std::io::Write as _;
use std::process::ExitCode;
use std::sync::mpsc;
use std::time::Duration;
use telltaled::{Clock, Sink};

/// The kernel file exposing the 1/5/15-minute load averages.
const LOADAVG_PATH: &str = "/proc/loadavg";

fn main() -> ExitCode {
    let vars: std::collections::HashMap<String, String> = std::env::vars().collect();
    let config = match telltaled::config::parse_config(&vars) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("telltaled: configuration error: {e}");
            return ExitCode::FAILURE;
        }
    };

    let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>();
    if let Err(e) = register_signals(shutdown_tx) {
        eprintln!("telltaled: signal setup: {e}");
        return ExitCode::FAILURE;
    }
    let mut sink = StdoutSink;
    let clock = RealClock;
    telltaled::run_loop(
        LOADAVG_PATH,
        &mut sink,
        &clock,
        config.interval_secs,
        &shutdown_rx,
    );
    ExitCode::SUCCESS
}

/// Spawn a background thread that listens for SIGTERM/SIGINT and sends a
/// shutdown notification on `tx` when the first signal arrives.
fn register_signals(tx: mpsc::Sender<()>) -> Result<(), Box<dyn std::error::Error>> {
    use signal_hook::consts::{SIGINT, SIGTERM};
    use signal_hook::iterator::Signals;

    let mut signals = Signals::new([SIGTERM, SIGINT])?;
    std::thread::spawn(move || {
        signals.forever().next();
        let _ = tx.send(());
    });
    Ok(())
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
