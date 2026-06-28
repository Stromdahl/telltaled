//! `telltaled` binary entry point — a thin shim over the `telltaled` library.
//!
//! M0 (issues/001): collect one load-average sample and write it to stdout, then
//! exit. The shim owns the only I/O — reading `/proc/loadavg` — and hands the
//! contents to the pure [`telltaled::loadavg::parse_loadavg`] core. Exits 0 on
//! success, non-zero (with the cause on stderr) if the read or parse fails. No
//! resident loop, scheduling, config, or transport yet — those are later
//! milestones.

use std::process::ExitCode;

/// The kernel file exposing the 1/5/15-minute load averages.
const LOADAVG_PATH: &str = "/proc/loadavg";

fn main() -> ExitCode {
    match run(LOADAVG_PATH) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("telltaled: {err}");
            ExitCode::FAILURE
        }
    }
}

/// Read, parse, and print one load-average sample from `path`. Errors carry the
/// path so a failure on stderr is self-explanatory. Taking the path as a
/// parameter (rather than reading the const directly) keeps both the success and
/// failure branches observable in tests without an argv/config surface.
fn run(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let contents = std::fs::read_to_string(path).map_err(|e| format!("reading {path}: {e}"))?;
    let sample =
        telltaled::loadavg::parse_loadavg(&contents).map_err(|e| format!("parsing {path}: {e}"))?;
    println!("{sample}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_succeeds_on_the_real_loadavg() {
        // /proc/loadavg is present on the Linux hosts telltaled targets.
        assert!(run(LOADAVG_PATH).is_ok());
    }

    #[test]
    fn run_errors_on_an_unreadable_path() {
        // A missing path surfaces an error, which main maps to a non-zero exit.
        assert!(run("/proc/telltaled-does-not-exist").is_err());
    }
}
