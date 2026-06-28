# PRD 0001 — MVP: load average → yggio

- Status: draft
- Date: 2026-06-28
- Related: ADR 0001 (name), ADR 0002 (HTTP transport via ureq+rustls), ADR 0003 (best-effort drop-and-log)

## Problem Statement

telltaled today is a one-shot probe (M0): it reads one `/proc/loadavg` sample,
prints it, and exits. That proves the parser but tells you nothing about a running
machine over time, and the data never leaves the host. To be a telltale — something
that surfaces a machine's hidden state where you can see it — it has to run
resident and deliver what it collects to a place you actually look. The operator
has no way to watch krypton's load from yggio, the work platform where fleet
telemetry already lives.

## Solution

A resident `telltaled` that, on each machine, samples the load average on a fixed
interval and pushes it to yggio over HTTP, so the operator sees a host's
1/5/15-minute load as live time-series in the yggio UI without logging into the box.

The MVP proves this end to end on one real host (krypton): one signal, sampled on a
loop, delivered to yggio, visible in the UI. It establishes the whole pipeline —
resident loop, pluggable transport's first concrete instance, configuration, device
identity, and a hardened system-service deployment — so that adding more signals
and more hosts afterward is incremental, not architectural. A second host (helium)
reuses the identical artifact once it exists.

## User Stories

1. As an operator, I want telltaled to run resident on a host and keep running
   across reboots and transient network outages, so that I have continuous coverage
   without babysitting it.
2. As an operator, I want a host's 1/5/15-minute load average to appear as
   time-series in yggio, so that I can watch load trends from the platform I already
   use.
3. As an operator, I want each host to report as its own yggio device, so that I can
   tell machines apart and chart or alert on them independently.
4. As an operator, I want a fresh-started daemon to report immediately rather than
   after a blind first interval, so that I get confirmation it is working at once.
5. As an operator, I want a network or yggio outage to cost only the samples taken
   during it — never the daemon itself — so that monitoring is not more fragile than
   the thing it monitors.
6. As an operator, I want to configure the endpoint, per-host secret, and interval
   without recompiling, so that I can deploy the same binary to every host.
7. As an operator, I want telltaled to run as a locked-down, unprivileged system
   service, so that a resident process with network egress carries minimal risk.
8. As a maintainer, I want the daemon to own its deployment contract (service unit +
   config template) in its own repo, so that a change to what it reads and how it
   runs lands in one place.

## Implementation Decisions

**Resident loop (M1).** A single-threaded, synchronous loop: sample → push → wait.
No async runtime. Default interval 60s (matches load-average granularity),
overridable. Ordering is **sample-first**: report once on startup, then wait. The
wait is **interruptible** — the loop blocks on a wakeable primitive (e.g. a
timed channel/condvar wait), not a bare sleep, so a SIGTERM handler can wake it for
prompt graceful shutdown rather than the signal idling until the interval elapses.
SIGTERM (sent by systemd on stop/restart) breaks the loop cleanly.

**Collection.** Reuses the existing pure `loadavg` core unchanged; the file read
stays in the thin shim. All three averages are collected.

**Transport (M2).** First concrete instance of the pluggable transport: yggio
generic HTTP push (ADR 0002). One `POST` per sample to the configured URL
(`…/http-push/generic?identifier=secret`), `content-type: application/json`, body:

```json
{"secret": "<host secret>", "load_1m": 0.42, "load_5m": 0.55, "load_15m": 0.61}
```

All three averages as separate numeric fields (each becomes a yggio time-series);
snake_case names; **no client timestamp** (yggio stamps `reportedAt` on receipt);
default `generic` node type. Client is `ureq` over rustls + webpki-roots
(`ureq = "3"`, default features). A short per-push HTTP timeout (~10s, < interval)
bounds worst-case loop wakeup. Verified against yggio source: the endpoint applies
no auth middleware — the `secret` is the lookup key selecting which existing device
to update — so the device must be provisioned first, and a 200 means accepted, not
persisted (fire-and-forget). This best-effort posture is ADR 0003: a failed push is
logged and dropped; the daemon never exits and never buffers/retries.

**Configuration (M2).** Environment variables only, no config-file parser:
`TELLTALED_YGGIO_URL`, `TELLTALED_YGGIO_SECRET` (sensitive), `TELLTALED_INTERVAL_SECS`
(default 60). Delivered via systemd `EnvironmentFile`. Missing required vars are a
clean startup error, not a panic.

**Device identity / provisioning.** One yggio device (iotnode) per host, each with
its own random `secret` (yggio charset `A–Z 0–9 - . _`, 8–128 chars), created
manually one-time in the yggio UI. Not automated in the daemon (would put personal
yggio credentials on every host). The secret lives only in the host's untracked env
file.

**Deployment (M3).** System service on every host under a dedicated unprivileged
`telltaled` user, with a hardened unit (e.g. `NoNewPrivileges`, `ProtectSystem`,
`ProtectHome`, `PrivateTmp`, `RestrictAddressFamilies` to inet). Single unit serves
all hosts. The daemon owns its deployment contract: the service unit and a
commented `*.env.example` template (placeholder values, no secret) live in a
`packaging/` directory in the telltaled repo. Installers (manual on krypton, Ansible
on helium) only copy the binary, copy the unit, render the secret, and enable.

**New dependency surface (deliberate).** `ureq` → rustls + crypto provider +
`webpki-roots`; `signal-hook` for SIGTERM. This is the project's jump from a
near-zero-dependency crate; each new license gets a deliberate `deny.toml` entry
with a reason, and `just deps` must pass. Build-time check: confirm which rustls
crypto provider lands (`aws-lc-rs` default needs a C toolchain/cmake; pin `ring` if
a self-contained build is preferred).

**New ADR likely surfaced.** The env-var-config + hardened-system-service
deployment model is a durable decision not yet captured in an ADR; candidate for a
future `decision-docs` hand-off (see Further Notes).

## Testing Decisions

The codebase's established seam is "pure, unit/property-testable core; I/O in a thin
shim" (see the existing `loadavg` parser and its proptest). The MVP keeps that seam
and adds three pure functions to test below the I/O boundary, plus one loop test:

- **Request body builder** — a pure function from `(LoadAvg, secret)` to the JSON
  body string/value. Test it asserts external behavior: the three `load_*` fields,
  the `secret`, snake_case names, and the *absence* of a timestamp. This is the
  highest-value new seam and mirrors how `parse_loadavg` is tested independently of
  the file read.
- **Config from environment** — a pure function from a string map to a validated
  config (or a clear error). Test required-var-missing errors and the interval
  default; no real `std::env` mutation in tests.
- **Loop control** — exercise the loop with an injected sink and an injected/fake
  clock so it runs a bounded number of iterations without real sleeping or network.
  Assert sample-first ordering, that a sink error does not stop iteration
  (best-effort, ADR 0003), and that the shutdown signal breaks the loop promptly.

Keep the actual `ureq` send and the real `/proc` read in thin shims, as today —
tested only by the existing liveness-style "runs against the real system" checks,
not by mocking HTTP. No new property test is required beyond the existing parser's
unless the body builder grows non-trivial logic.

## Out of Scope

- MQTT and any other transport (HTTP push only).
- Buffering, retry, or replay of undelivered samples (ADR 0003).
- A config file (TOML/serde) and per-collector settings.
- Automated device provisioning / first-boot registration.
- Additional signals — temperature, storage, anything beyond load average.
- A client-supplied timestamp.
- helium deployment is **post-MVP** (M4): it has no OS yet. The MVP designs for it
  (identical artifact, sops-rendered secret, Ansible installer) but does not block
  on it.
- The threat-model document. AGENTS.md flags it for when the first network
  transport lands; M2 is arguably that trigger, but writing it is not part of this
  PRD.

## Further Notes

- **Milestone sequencing** (each ends with `just check` green, independently
  shippable): **M1** resident loop to stdout (loop, interval, interruptible sleep,
  SIGTERM) — no network; **M2** HTTP transport + env config + best-effort push,
  verified live against a provisioned krypton device; **M3** packaging + hardened
  unit + install on krypton, confirmed in the yggio UI; **M4** (post-MVP) helium via
  Ansible reusing the same artifact.
- **Contract verification status:** the yggio HTTP contract is source-verified
  against the integration's route + interpreter code. It has **not** yet been
  confirmed with a live POST; first live validation is M2/M3 (a sample appearing in
  the UI).
- **Decision-docs hand-off available:** a future ADR (≈0004) for the env-var-config
  + hardened-system-service deployment model would capture the one durable decision
  this PRD relies on that isn't yet in an ADR.
