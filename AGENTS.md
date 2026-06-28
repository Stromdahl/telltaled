# AGENTS.md — how telltaled is built

Instructions for any coding assistant (Claude Code, etc.) working in this repo.

## What telltaled is

`telltaled` is a system telemetry daemon: it runs resident on each machine in the
fleet, collects system statistics (load, temperature, storage, and as much else as
can be gathered), and reports them in a pluggable serialization format over a
pluggable transport (stdout, HTTP, MQTT, …). The name follows ADR 0001 — a
"telltale" surfaces an otherwise-hidden condition.

The single constraint that outranks features and convenience: **low host
overhead**. Measuring a machine must never meaningfully perturb it — footprint,
allocations, syscalls, and wakeups are budgeted, and quiescence beats feature
richness. A monitor you have to worry about isn't one. When a choice trades
overhead for capability, overhead wins; if you must spend it, spend it on purpose.

## Working principles
- **Work from first principles; validate, don't assume.** Check how the code
  actually behaves — read the source, run it, write a test — before building on a
  belief about it. An assumption you didn't verify is a bug you haven't found yet.
- **Treat your training knowledge as probably stale.** Library APIs, flags,
  versions, and defaults drift. Before relying on a fact, confirm it against
  current sources — official docs, `--help`, the installed source, release notes —
  not memory. When uncertainty remains, say so rather than asserting.

> Security: not yet a first-class layer. `telltaled` is a resident daemon with
> network egress and a plausible future inbound scrape endpoint, so this needs a
> threat model — revisit when the first network transport lands (add
> `docs/THREAT_MODEL.md` + the security-invariants section then). When collectors
> begin shelling out for sensor data, lock the subprocess boundary with a
> `clippy.toml` `disallowed-methods` gate.

## Workflow
- **TDD**: every issue starts with a failing test; implement; refactor. Core
  invariants get `proptest` property tests, not just examples — plus a liveness
  test so a vacuous "reject everything" implementation can't pass.
- **Issues**: tracked on **GitHub Issues** (`gh issue list` / `gh issue create`).
  Create an issue before starting work and reference it as `#N` in commits and
  PRs. Large issues are decomposed into child **sub-issues** carrying a
  self-contained execution brief (entry points, steps, verify, acceptance
  criteria).
- **Grabbing work** (agent queue): a sub-issue/issue labelled `ready` is an
  unblocked brief free to pick up. To grab it, swap `ready` → `in-progress` (the
  lock) before starting; on completion, close it and move any dependent issue
  whose blockers are now all closed from `blocked` → `ready`. `blocked` means
  unmet dependencies — don't start. Status labels are queue bookkeeping: commit
  them promptly and directly on `main` (exempt from "branch first") so concurrent
  sessions see an accurate queue.
- **Milestones**: work climbs a milestone ladder (M0, M1, …) — each milestone a
  thin, shippable slice that ends with the gate green, not a big-bang. The current
  milestone is noted in the README.
- **Docs-first**: record decisions as ADRs in `docs/adr/` (format in
  `docs/adr/README.md`). Keep design docs current as the code changes.
- **Commits**: a single conventional-commit subject line, no body, no AI
  co-author trailer — full convention in `docs/COMMIT_STYLE.md`. Reference
  GitHub issue ids (`#N`). Commit **proactively** at coherent stopping points (one
  cohesive, non-broken change per commit; never a broken state or a secret).
  Pushing stays ask-first.

## Quality guardrails
The local gate is **`just check`** (fmt → clippy `-D warnings` → tests → file
length). Run it before committing; it must be green. `just --list` shows all
recipes. Lints are `warn`-level so the editor surfaces them without blocking
iteration, and `just check` turns them into failures.

**Enforced (fail the gate):**
- `#![forbid(unsafe_code)]` (workspace lint) — relax deliberately if you take on
  FFI or similar; otherwise no `unsafe`.
- No `unwrap`/`expect`/`panic!`/`todo!`/`dbg!` in production code (fine in tests).
- Function length ≤ 60 lines · parameters ≤ 5 · nesting depth ≤ 3 · file ≤ 400
  lines. Thresholds live in `clippy.toml`; tune them there, with a reason.
- `cargo fmt` clean (deterministic formatting > any duplication scanner).
- Supply chain: `just deps` (cargo-deny advisories/licenses/bans/sources +
  cargo-audit). A new license or git source fails until added deliberately.

**Advisory / not natively enforceable in Rust — don't assume coverage you lack:**
- Cognitive complexity: clippy's metric *severely* undercounts (the threshold is
  calibrated to its scale, not the literal table number). Directional.
- Cyclomatic complexity & code duplication: no native lint — judgment, not a gate.

**On-demand (install with `just bootstrap`):** `just cov` / `cov-gate` (≥80%
lines), `just mutants` (mutation score — the real "do tests catch bugs?" signal;
prioritise it on whatever module holds your core invariants), `just api`
(public-API surface), `just msrv` (build on the pinned toolchain).

When a guardrail fires on code that is genuinely clearer as-is, `#[allow(...)]`
it *with a reason* — the number serves readability, not the other way around.

## Layout
- `crates/telltaled/` — the daemon: one package exposing the testable library
  core (`src/lib.rs`) and a thin binary shim (`src/main.rs`). The workspace is
  kept deliberately — "arbitrary serialization over arbitrary transport" (ADR 0001)
  will likely split collector/transport/format into their own crates.
- `justfile` — the check gate and tooling recipes. `clippy.toml` / `deny.toml`
  hold the guardrail thresholds and supply-chain policy; `[workspace.lints]` in
  the root `Cargo.toml` enables the lints (inherited via `[lints] workspace = true`).
- `docs/adr/` — architecture decision records. Issues live on GitHub Issues.
