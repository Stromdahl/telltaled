# 0003 — Best-effort telemetry: drop-and-log, no buffering

- Status: accepted
- Date: 2026-06-28

## Context

Once telltaled samples on a resident loop and pushes each sample to a network sink
(ADR 0002), pushes will sometimes fail: the host loses network, the sink returns an
error, or a connection hangs. The loop needs a defined stance on what to do with a
sample it cannot deliver, and on whether a delivery failure should affect the
daemon's liveness.

The forces:

- The overriding constraint is low host overhead. Buffering undelivered samples to
  disk or memory, and replaying them, adds persistent state, I/O, and wakeups — the
  exact costs AGENTS.md tells us to budget against.
- The first signal (load average) is a continuously-resampled gauge, not a
  discrete event. A point missed at one interval is superseded by the next; its
  information value decays to near zero within an interval or two.
- The daemon runs under systemd with a restart policy. If a transient push failure
  killed the process, the supervisor would flap it on every network blip.

## Decision

We will treat the sink as **best-effort**. On a failed push, telltaled logs one
line (to stderr → journal) and continues to the next sample. It does **not** exit,
does **not** retry within the interval, and does **not** buffer, queue, or replay
undelivered samples. Pushes carry a short HTTP timeout, well under the sampling
interval, so a hung connection cannot stall or overlap the loop.

## Consequences

- A network or sink outage costs only the samples taken during it; the daemon keeps
  running and resumes reporting automatically when the sink returns. Liveness is
  decoupled from sink availability.
- yggio's own write is fire-and-forget (a 200 means accepted, not persisted), so
  even a "successful" push is best-effort end to end — consistent with this stance.
- Gaps in the time-series are expected and acceptable for gauge signals. If a future
  signal is event-like (where each sample is irreplaceable), this decision must be
  revisited for that signal — likely a new ADR introducing bounded buffering or a
  durable transport (e.g. MQTT QoS), scoped to that need rather than applied
  globally.
- The short per-push timeout bounds worst-case loop wakeup duration; choosing it too
  long would let a black-hole connection wedge the loop, too short would drop
  deliverable samples on a slow link.
