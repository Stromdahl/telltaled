# ADR-0001: Name the system telemetry daemon `telltaled`

- Status: accepted
- Date: 2026-06-28
- Deciders: Stromdahl

## Context and Problem Statement

We are building a daemon that runs on each machine in the fleet, collects system statistics (load, temperature, storage, and as much additional data as can be gathered), and reports them in an arbitrary serialization format over an arbitrary transport (plain text, HTTP, MQTT, etc.). The project needs a name that is descriptive, available, and free of confusing collisions in our problem domain (system telemetry / monitoring).

## Decision Drivers

- Should evoke "surfacing the hidden state of a machine."
- Must follow daemon naming conventions, since it runs as a resident service.
- Should avoid hard collisions with existing tools in the telemetry/monitoring space.
- Should be reasonable to type and recognizable in logs and unit files.

## Considered Options

- `telltaled`
- `telltale-stats`
- `telltale` (bare)
- Alternatives: `beacon`, `sentinel`, `pulse`, `heimdall`, `huginn`

## Decision Outcome

Chosen option: **`telltaled`**.

A "telltale" is an indicator that reveals an otherwise-hidden condition — originally a fabric strip on a sail that reveals airflow, later reused in aviation/engineering for warning indicators. This maps directly onto the daemon's purpose: surfacing the hidden state of a machine. The `-d` suffix follows the established daemon convention (`sshd`, `systemd`, `containerd`), correctly signalling a resident service rather than a one-shot probe.

### Consequences

- Good: name is apt, conventional for a daemon, and self-documenting in `systemd` unit files (`telltaled.service`).
- Good: avoids the collisions found in the bare `telltale` namespace.
- Bad: the bare term "telltale" is crowded on GitHub (predominantly Telltale Games modding tooling), which adds search noise; the `-d` suffix mitigates but does not eliminate this.
- Neutral: `telltaled` is visually ambiguous (reads as either "telltale-dee" the daemon, or "telltaled" the past tense). Cosmetic only.
- Follow-up: crate name availability on crates.io has **not** yet been verified — see "Confirmation" below.

### Confirmation

- Conventions: unit `telltaled.service`; config under `/etc/telltaled/` (or `~/.config/telltaled/`); binary `telltaled`.
- **Open action:** verify `telltaled` (and fallback `telltale-stats`) availability on crates.io before first publish.

> Update 2026-06-28: `telltaled` and `telltale-stats` both confirmed available on crates.io (registry API returns 404). Open action resolved.

## Pros and Cons of the Options

### `telltaled`

- Good: follows daemon convention; signals resident service.
- Good: sidesteps bare-namespace collisions.
- Neutral: minor visual ambiguity in the name.

### `telltale-stats`

- Good: fully self-describing; cleanest collision avoidance.
- Bad: longer to type; reads as a descriptive package name rather than a daemon; `-stats` is generic/redundant.

### `telltale` (bare)

- Good: shortest, cleanest as a CLI verb.
- Bad: crowded namespace; closest real collision is `ajmyyra/telltale`, an HTTP echo/debug tool in the same infra-adjacent space — direct enough to cause confusion.

### Other names (`beacon`, `sentinel`, `pulse`, `heimdall`, `huginn`)

- Bad: all are used by well-known monitoring/automation projects; rejected on collision grounds.
