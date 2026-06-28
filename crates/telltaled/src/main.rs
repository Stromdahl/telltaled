//! `telltaled` binary entry point — a thin shim over the `telltaled` library.
//!
//! Scaffolding only: prints a startup banner. Real collection/transport wiring
//! lands with the milestone slices tracked under `issues/` (start at M0).

fn main() {
    println!("telltaled {}", telltaled::version());
}
