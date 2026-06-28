# telltaled

A system telemetry daemon. It runs resident on each machine in a fleet, collects
system statistics (load, temperature, storage, and more), and reports them in a
pluggable serialization format over a pluggable transport (stdout, HTTP, MQTT, …).

A *telltale* is an indicator that surfaces an otherwise-hidden condition — a strip
on a sail that reveals airflow, a warning lamp on a dashboard. `telltaled` does that
for a machine.

**Overriding constraint: low host overhead.** Measuring a machine must never
meaningfully perturb it; when a choice trades footprint for capability, footprint wins.

## Status

**M0 shipped:** collects one `/proc/loadavg` sample, writes it to stdout, exits. Work
climbs a milestone ladder of thin shippable slices; M1 (a resident sampling loop) is next.

## Layout

- `crates/telltaled/` — the daemon: a testable library core (`src/lib.rs`) plus a thin
  binary shim (`src/main.rs`).
- `docs/adr/` — architecture decision records.
- `justfile`, `clippy.toml`, `deny.toml` — the quality gate and its thresholds.

## The gate

`just check` = `fmt-check` → `lint` (clippy `-D warnings`) → `test` → `file-length`.
Run it before every commit. `just --list` shows on-demand recipes (`cov`, `mutants`,
`api`, `msrv`, `deps`, `bootstrap`).

See [`AGENTS.md`](AGENTS.md) for the guardrail rationale and the TDD + docs-first workflow.

## Conventions

- **Daemon:** binary `telltaled`; systemd unit `telltaled.service`; config under
  `/etc/telltaled/` (or `~/.config/telltaled/`).
- **Issues:** tracked in GitHub Issues (`gh issue list`); reference as `#N`.
- **Commits:** one conventional-commit subject line — see [`docs/COMMIT_STYLE.md`](docs/COMMIT_STYLE.md).

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.
