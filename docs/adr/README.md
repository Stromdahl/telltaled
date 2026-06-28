# Architecture Decision Records — Specification

**Version: 0.0.1**

A lightweight convention for recording architectural decisions as short,
immutable markdown files that live with the code. An ADR captures one decision:
the context that forced it, the choice made, and the consequences accepted — so
the *why* survives long after the decision stops feeling obvious.

## Why

- **The reasoning outlives the author.** Six months on, "why is it like this?"
  has a written answer instead of an archaeology project.
- **Decisions, not documentation.** An ADR records a *choice at a point in time*,
  not the current state of the system — so it stays true even as code moves.
- **Plain text, in the repo.** Diffs, history, and grep all work. AI assistants
  and humans read and write the same format. No tool to install.

The trade-off: ADRs are not living docs. They're not edited to track reality —
they're superseded. If you want a always-current design overview, that's a
different document.

## When to write one

Write an ADR when a decision is **costly to reverse**, **shapes the structure**,
or will otherwise prompt a future "why on earth is it like this?" — choosing a
boundary, a storage model, a protocol, a dependency you'll be married to, a hard
constraint. Skip it for routine, easily-reverted choices; not every commit needs
a paper trail.

## Layout

```
<repo>/
└── docs/
    └── adr/
        ├── README.md                 # this spec, or a pointer to it
        ├── 0001-some-decision.md
        └── 0002-another-decision.md
```

## File naming

`NNNN-short-slug.md`

- `NNNN` — zero-padded four-digit id, monotonically increasing from `0001`.
- `short-slug` — kebab-case, derived from the title. A stable hint, not a source
  of truth.
- The basename **never changes** after creation. Even a superseded or abandoned
  ADR keeps its file and its number.

## Format

Each ADR has these sections, in order:

- **Status** — `proposed` | `accepted` | `superseded by NNNN` | `deprecated`.
- **Date** — `YYYY-MM-DD` the status last changed.
- **Context** — the forces at play: what makes this decision necessary now, the
  constraints, the tension being resolved. State the problem, not the solution.
- **Decision** — what was chosen, stated plainly ("We will …").
- **Consequences** — what becomes true as a result, good and bad: new
  obligations, things now ruled out, follow-ups, trade-offs accepted.

## Conventions

- **Immutable once accepted.** Don't rewrite history. To change a decision, write
  a *new* ADR and set the old one's `Status` to `superseded by NNNN`.
- **Never reuse a number**, even for an abandoned proposal. The number is spent.
- **Keep it to a page.** If an ADR is sprawling, it's probably two decisions —
  split it.
- **Reference ADRs by number** (`ADR 0003`) from code comments, commit messages,
  and issues.

## Adoption checklist (new project)

1. `mkdir -p docs/adr`
2. Copy this file to `docs/adr/README.md` (or write a short pointer to it).
3. Optionally drop the template below in as `docs/adr/0000-template.md`.
4. Point contributors (human or AI) at `docs/adr/` from `AGENTS.md` / `README`.
5. Write `0001` for the first real decision.

## Template

Drop this into `docs/adr/NNNN-slug.md` and fill in:

```markdown
# NNNN — <Decision title>

- Status: proposed
- Date: YYYY-MM-DD

## Context
<The forces at play: what makes this decision necessary now, the constraints,
the tension being resolved. State the problem, not the solution.>

## Decision
<What we will do, stated plainly. "We will …">

## Consequences
- <What becomes true as a result — new obligations, things now ruled out,
  follow-ups, trade-offs accepted.>
```
