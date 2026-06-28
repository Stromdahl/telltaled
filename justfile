# Task runner. `just` is a command runner — cargo owns the build graph, so this
# is purely for running checks. `just --list` shows everything. See AGENTS.md.

# Max physical lines per .rs file (table target 300-500).
file-line-limit := "400"

# Default: list recipes.
default:
    @just --list

# THE GATE. Run before committing; fails on any guardrail violation.
check: fmt-check lint test file-length
    @echo "✓ guardrails pass (fmt, clippy, tests, file length)"

# Reject unformatted code (no writes).
fmt-check:
    cargo fmt --all -- --check

# Apply formatting.
fmt:
    cargo fmt --all

# Clippy with the guardrail lints; warnings (incl. thresholds) fail the gate.
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Tests (unit + proptest + doctests).
test:
    cargo test --all-features

# Fail if any .rs file exceeds the line limit (clippy can't check whole-file length).
file-length:
    #!/usr/bin/env bash
    set -euo pipefail
    limit={{file-line-limit}}
    fail=0
    while IFS= read -r f; do
        n=$(wc -l < "$f")
        if (( n > limit )); then
            printf '✗ %s: %d lines (limit %d)\n' "$f" "$n" "$limit"
            fail=1
        fi
    done < <(find . -type d -name target -prune -o -type f -name '*.rs' -print)
    (( fail == 0 )) && echo "✓ all .rs files ≤ $limit lines"
    exit $fail

# --- On-demand checks (install the tools with `just bootstrap`) ---

# Coverage report (cargo-llvm-cov). Pass extra args through, e.g. `just cov --html`.
cov *ARGS:
    cargo llvm-cov --all-features {{ARGS}}

# Coverage gate: fail under 80% line coverage (table target).
cov-gate:
    cargo llvm-cov --all-features --fail-under-lines 80

# Detail (separated by a blank line so `just --list` keeps the one-liner below):
# cargo-mutants is the real "do the tests catch bugs?" signal — slow. Prioritise
# it on whatever module holds your core invariants.

# Mutation testing (cargo-mutants) — slow; measures whether tests catch bugs.
mutants *ARGS:
    cargo mutants {{ARGS}}

# Public API surface (cargo-public-api) — review what contracts changed.
api:
    cargo public-api

# Supply-chain: dependency policy (cargo-deny) + RUSTSEC advisories (cargo-audit).
deps:
    cargo deny check
    cargo audit

# Detail (blank line keeps the one-liner below as the `just --list` description):
# If a crate needs a higher floor than the workspace default, give it its own
# `rust-version` and add a line here that checks just that crate on its toolchain.

# Verify the workspace builds on the pinned MSRV (rust-version in Cargo.toml).
msrv:
    cargo +1.85.0 check --workspace --all-features

# Install the on-demand tooling (cargo-deny and cargo-audit are assumed present).
bootstrap:
    rustup component add llvm-tools-preview
    cargo install cargo-llvm-cov cargo-mutants cargo-public-api
