# telltaled

A system telemetry daemon. It runs resident on each machine in a fleet, collects
system statistics (load, temperature, storage, and as much else as can be
gathered), and reports them in a pluggable serialization format over a pluggable
transport (stdout, HTTP, MQTT, …).

A *telltale* is an indicator that surfaces an otherwise-hidden condition — a strip
on a sail that reveals airflow, a warning lamp on a dashboard. `telltaled` does
that for a machine. The name and `-d` daemon suffix are settled in
[`docs/adr/0001`](docs/adr/0001-name-the-system-telemetry-daemon-telltaled.md).

**Overriding constraint: low host overhead.** Measuring a machine must never
meaningfully perturb it. When a choice trades footprint for capability, footprint
wins. See [`AGENTS.md`](AGENTS.md) for how that constraint is applied.

## Status

**Milestone M0 — shipped.** `telltaled` collects one `/proc/loadavg` sample and
writes it to stdout, then exits (see closed [`issues/001`](issues/001-m0-one-shot-load-sample-to-stdout.md)).
Work climbs a milestone ladder (M0, M1, …), each a thin shippable slice that ends
with the gate green; M1 (a resident sampling loop) is next.

## Layout

- `crates/telltaled/` — the daemon. One package: a testable library core
  (`src/lib.rs`) plus a thin binary shim (`src/main.rs`). The workspace is kept
  deliberately — collector / transport / format will likely become their own
  crates as the "arbitrary transport, arbitrary format" goal firms up.
- `docs/adr/` — architecture decision records (`README.md` is the format spec).
- `issues/` — the in-repo, file-based issue tracker (`README.md` is the spec).
- `justfile`, `clippy.toml`, `deny.toml` — the quality gate and its thresholds.

## The gate

`just check` = `fmt-check` → `lint` (clippy `-D warnings`) → `test` →
`file-length`. Run it before every commit; keep it green. `just --list` shows the
on-demand recipes (`cov`, `mutants`, `api`, `msrv`, `deps`, `bootstrap`). See
[`AGENTS.md`](AGENTS.md) for the full guardrail rationale and the TDD + issues +
docs-first workflow.

## Conventions

- **Daemon:** binary `telltaled`; systemd unit `telltaled.service`; config under
  `/etc/telltaled/` (or `~/.config/telltaled/`).
- **Commits:** one conventional-commit subject line, referencing `issues/NNN`
  where relevant — see [`docs/COMMIT_STYLE.md`](docs/COMMIT_STYLE.md).
