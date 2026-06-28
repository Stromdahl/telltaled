# In-Repo Issue Tracker — Specification

> **Retired.** This repo has migrated to **GitHub Issues**
> (`gh issue list` / `gh issue create`; reference as `#N`). The files in this
> directory are kept for git history only — do not file new issues here. The
> spec below is preserved as a record of the former workflow.

**Version: 0.0.2**

A lightweight, file-based issue tracker that lives inside the repository.
Each issue is one markdown file with YAML frontmatter for machine-readable
state and a free-form body for human context. No external service, no
database, no separate tool — `ls`, `grep`, and a text editor are enough.

## Why

- **Issues live with the code.** Branching, diffing, and history all work.
- **No account, no API.** Survives forks, mirrors, offline work, and the
  death of any particular SaaS.
- **Plain text.** AI assistants and humans read and write the same format.
- **No tool to install.** The "tracker" is just a directory convention.

The trade-off: no cross-repo views, no notifications, no rich linking.
For projects where those matter, use GitHub Issues / Linear / etc.

## Layout

```
<repo>/
└── issues/
    ├── README.md                  # this spec, or a pointer to it
    ├── 001-some-bug.md            # every issue — open or closed — lives here
    ├── 002-other-bug.md
    └── 014-some-feature.md
```

Every issue lives directly in `issues/`, regardless of status. **Files are
never moved.** The `status` frontmatter field is the single source of truth for
open vs. closed. Pinning each file to a fixed path preserves git traceability —
`git log` and `git blame` follow one stable path, and every `issues/NNN`
reference (in commits, PRs, other issues) stays valid for the life of the repo.

## File naming

`NNN-short-slug.md`

- `NNN` — zero-padded three-digit ID, monotonically increasing.
- `short-slug` — kebab-case, derived from the title. Stable hint, not a
  source of truth.
- The basename **never changes** after creation, and the file is **never
  moved**. State is read entirely from the `status` frontmatter field.

## Frontmatter schema

Every issue file starts with a YAML frontmatter block:

```yaml
---
title: Human-readable title
status: open | in-progress | closed
priority: high | medium | low
created: YYYY-MM-DD
closed: null              # YYYY-MM-DD when resolved, else null
labels: [bug, backend]    # free-form tags
---
```

Issues do not carry a per-file spec-version field — the spec version is
a property of the spec document, not of each artifact. Projects pin a
spec version by copying this file (with its version line at the top)
into their own repo. When the spec changes incompatibly, bump the
version line here first; projects upgrade by re-syncing the file.

Field semantics:

- **title** — one line, < 80 chars. Should stand alone in a list view.
- **status** — the source of truth for open vs. closed. `in-progress` is
  optional; treat it as a flavor of open.
- **priority** — coarse triage signal. Three levels is enough.
- **created** — set on creation, never changed.
- **closed** — set when `status` flips to `closed`, cleared on reopen.
- **labels** — free-form. Common ones: `bug`, `feature`, `security`,
  area tags (`backend`, `frontend`), milestone tags (`phase-5`).

## Body

Three conventional sections, in this order:

```markdown
## Description

What and why. One or two paragraphs.

## Acceptance criteria

- [ ] Concrete, checkable conditions for "done".
- [ ] Prefer a checklist; check items off as work lands.

## Progress

Newest first. Notable steps, decisions, blockers, links to commits or
PRs. Optional — many issues never need it.
```

Sections may be omitted if empty. Add others as needed (`Notes`,
`References`), but keep the three above as the canonical skeleton.

## Operations

### Create

1. Pick the next free `NNN`.
2. Write `issues/NNN-slug.md` with the frontmatter and body.
3. Commit. The commit message should explain the *why*, not restate the
   title.

### Update

Edit the file. Common changes:

- Flip `status: open` → `status: in-progress` when work starts.
- Check off acceptance criteria as they land.
- Append to `Progress` for non-obvious decisions or blockers.

### Close

Update the frontmatter **in place** — the file does not move:

```yaml
status: closed
closed: YYYY-MM-DD
```

Optionally add a final `Progress` entry summarizing how it was resolved and
the commit/PR that closed it.

**Never rename, move, or delete an issue file.** Its path is fixed for the life
of the repo; only the `status` field changes. This is deliberate: relocating a
file on close churns git history and breaks every reference to its old path.

### Reopen

Set `status` back to `open` (or `in-progress`) and set `closed` back to `null`.
Nothing moves.

## Listing

Every issue lives in `issues/`; filter by the `status` field, not by directory:

```sh
# All issues
ls issues/[0-9]*.md

# Open / in-progress only
grep -l '^status: \(open\|in-progress\)' issues/[0-9]*.md

# Closed only
grep -l '^status: closed' issues/[0-9]*.md

# Issues with a label
grep -l '^labels:.*\bbug\b' issues/[0-9]*.md

# Title + status, one line per issue
awk '/^title:/{t=$0} /^status:/{print FILENAME, $2, t}' issues/[0-9]*.md
```

The `status` frontmatter field is the source of truth.

If a project grows enough to want a richer view, write a small script
(`issues/list.sh` or similar) that parses the frontmatter. Don't
introduce a dependency just for listing.

## Conventions

- **One issue per file.** Don't bundle.
- **Never reuse IDs.** Even if an issue is closed as a duplicate or
  invalid, its number is spent.
- **Reference issues as `issues/NNN`** in commit messages, PR descriptions,
  and other issues. The path is fixed, so the reference never rots.
- **Keep titles under 80 characters.** Body is for detail.
- **Don't move or delete issues.** A closed issue stays at its path; the
  closed history is the point.
- **Create issues proactively.** When a bug surfaces or a gap is found,
  file it in the same change rather than letting it evaporate.

## Adoption checklist (new project)

1. `mkdir issues/`
2. Copy this file to `issues/README.md` (or write a short pointer to it).
3. Add a one-paragraph note in `agents.md` / `CLAUDE.md` / `CONTRIBUTING.md`
   pointing future contributors (human or AI) at `issues/README.md`.
4. File the first issue. Now there's a tracker.

## Issue template

Drop this into `issues/NNN-slug.md` and fill in:

```markdown
---
title:
status: open
priority: medium
created: YYYY-MM-DD
closed: null
labels: []
---

## Description

## Acceptance criteria

- [ ]
```
