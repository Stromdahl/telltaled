# Commit Message Style — Specification

**Version: 0.0.1**

A small, opinionated convention for git commit messages: a single
conventional-commit subject line, no body, no AI attribution. Optimised for a
log that's fast to scan and `git log --oneline`-friendly.

## Why

- **One line scans.** A subject-only log reads top-to-bottom without expanding
  anything. The history *is* the changelog.
- **The prefix sorts the world.** `feat:` / `fix:` / `security:` tells you the
  shape of a change before you read it, and groups related work.
- **The issue id ties it back.** A change references the issue that motivated it,
  so the *why* is one `grep` away (see the in-repo issue tracker spec).

## The rule

```
<type>: <imperative subject, lower-case, no trailing period>
```

- **Exactly one subject line.** No body paragraphs, no blank-line-then-prose.
  Pass a single `-m "…"` to `git commit`; never a HEREDOC body.
- **Keep it under ~70 characters.** It has to fit a terminal column and a PR list.
- **Imperative mood.** "add X", "fix Y" — not "added"/"fixes"/"adding".
- **Match the existing log's prefix style.** If a repo already uses `area:`
  prefixes (`audit:`, `cli:`), follow that; otherwise use the types below.

## Types

The conventional-commits subset in use:

- `feat:` — a new capability.
- `fix:` — a bug fix.
- `test:` — tests only (new or changed), no behaviour change.
- `refactor:` — restructuring with no behaviour change.
- `docs:` — documentation, comments, ADRs.
- `chore:` — tooling, deps, scaffolding, housekeeping.
- `security:` — a security-relevant change (call it out explicitly).

Add area prefixes (`cli:`, `core:`) where a repo's log already uses them.

## Referencing issues

When a commit relates to a tracked issue, reference it as `issues/NNN` in the
subject (or trailing, if it fits): `feat: durable audit backend (issues/009)`.
This is a logical reference — the file may be open or closed, but the id is
unique and searchable.

## No AI attribution

**Never add a `Co-Authored-By: <AI assistant>` trailer** (or any other co-author
trailer attributing the commit to an assistant). An assistant is a tool, not a
collaborator; the commit message carries the subject and nothing else.

## Examples

```
feat: add token-bucket rate limiter to the public API (issues/042)
fix: reject expired sessions instead of silently refreshing them
security: require auth before any write endpoint is reachable
test: property test that the parser round-trips every input
chore: pin the toolchain and wire up the check gate
docs: record the storage-engine choice (ADR 0003)
```

## Adoption checklist (new project)

1. Note the convention in `AGENTS.md` (one line + a pointer to this spec).
2. If using a commit-msg hook, enforce the subject length and prefix there.
3. That's it — there's nothing to install.
