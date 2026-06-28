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
//! This is scaffolding. The first real slice is tracked in `issues/001`; per the
//! TDD workflow, it starts with a failing test.

/// The running daemon's version, taken from the crate metadata.
#[must_use]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_reported() {
        assert!(!version().is_empty());
    }
}
