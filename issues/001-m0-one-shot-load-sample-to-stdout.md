---
title: M0 — one-shot load-average sample to stdout
status: closed
priority: high
created: 2026-06-28
closed: 2026-06-28
labels: [feature, phase-0]
---

## Description

The first shippable slice (M0): a `telltaled` invocation collects one telemetry
sample and writes it to stdout, then exits. This proves the end-to-end shape —
collect → represent → emit — at the lowest possible cost before any resident loop,
config, serialization layer, or network transport exists.

Load average is the cheapest first signal on Linux: a single read of
`/proc/loadavg`. Parsing must be a pure function (input: the file's contents;
output: the three averages), so it is unit- and property-testable without touching
the real system — keeping with the **low host overhead** constraint (no extra
syscalls, no background work in M0).

Out of scope for M0: a resident loop, scheduling, config files, pluggable
serialization, and any network transport. Those are later milestones.

## Acceptance criteria

- [x] A pure parser turns `/proc/loadavg` contents into the 1/5/15-minute averages,
      returning a typed error (no panic/unwrap) on malformed input.
- [x] Unit tests cover a known-good line and representative malformed inputs.
- [x] A `proptest` property test asserts the parser never panics on arbitrary
      input, plus a liveness test so a vacuous "always error" parser can't pass.
- [x] `telltaled` run as a binary reads `/proc/loadavg`, prints the sample on one
      line to stdout, and exits 0 (non-zero on read/parse failure).
- [x] `just check` is green.

## Progress

- 2026-06-28: filed alongside project scaffolding (ADR 0001). Not yet started —
  per the TDD workflow, begins with a failing parser test.
- 2026-06-28: implemented. `crates/telltaled/src/loadavg.rs` holds the pure
  parser (`split_whitespace`, first three fields → `f64`, typed `ParseError`)
  plus unit tests and two `proptest` properties (never-panics + round-trip
  liveness); `main.rs` reads `/proc/loadavg` and prints `load 1m 5m 15m` via an
  `ExitCode` shim. TDD red→green confirmed; `just check` and `just deps` green.
  Closes M0.
- 2026-06-28: migrated to GitHub Issues as
  [#1](https://github.com/Stromdahl/telltaled/issues/1). This file is now a
  historical record only; the in-repo tracker is retired.
