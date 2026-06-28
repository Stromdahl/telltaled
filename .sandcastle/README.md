# Sandcastle orchestration for telltaled

Runs AI coding agents in an isolated Docker sandbox against telltaled's
**`ready`-flag** task workflow: pick an unblocked `ready` sub-task issue → lock it
`in-progress` → implement the brief on a branch → open a PR → iterate on review
feedback → you merge.

Built on [`@ai-hero/sandcastle`](https://github.com/mattpocock/sandcastle). The Rust
repo stays the source of truth; this is just the agent harness.

## Files

| File | Purpose |
|------|---------|
| `run.mts` | Fresh run: select a `ready` issue, lock it, implement the brief, open a PR. |
| `iterate.mts` | Apply review feedback to an open PR (branch re-entry; `--resume` to also continue the agent's session). |
| `lib.mts` | Shared host-side helpers (gh/git, state, providers). |
| `prompt.md` | Fresh-run brief — injects the issue body, points at `AGENTS.md` + `just check`. |
| `iterate-prompt.md` | Iteration brief — injects the review feedback. |
| `Dockerfile` | Sandbox image: Claude Code CLI + Rust toolchain + `just`. No `gh` (GitHub stays host-side). |
| `.env.example` | Template for agent auth (`CLAUDE_CODE_OAUTH_TOKEN` / `ANTHROPIC_API_KEY`). |

## One-time setup

```bash
npm install                                  # installs sandcastle + tsx
cp .sandcastle/.env.example .sandcastle/.env # then fill in your token (claude setup-token)
npm run sandcastle:build-image               # builds the Docker image (Rust toolchain — slow first time)
gh auth status                               # the driver uses host gh for labels + PRs
```

## The lifecycle

You own the two ends (writing the brief, reviewing the PR); the harness owns the
mechanical middle.

```
   (has unmet deps)        (you mark grabbable)        (driver grabs it)
      blocked  ───────────────►  ready  ───────────────►  in-progress
                                   ▲                          │
                                   │   run fails              │ commit → push → PR
                                   └──────────────────────────┤
                                                              ▼
                                              PR open ──► review ──► (iterate*) ──► you merge
```

1. **Shape the task (you).** Write a self-contained execution brief as a sub-issue
   (Description · Entry points · Steps TDD · Verify · Acceptance · Out of scope),
   label it `ready` once it's genuinely unblocked. The agent sees only the brief +
   `AGENTS.md` — brief quality is the main lever.
2. **Dispatch (harness):** `npm run sandcastle [-- N]`. Locks the issue, cuts
   `agent/issue-<N>` **from `origin/main`**, implements under `just check`, pushes,
   opens a PR.
3. **Review (you).** The gate passed in the sandbox; review for correctness/design.
4. **Iterate (harness):** `npm run sandcastle:iterate -- N "feedback"` adds commits
   to the open PR. No longer manual.
5. **Merge (you).** Merge the PR (`Closes #N`); pull `main`; flip dependent issues
   `blocked`→`ready`.

## Run a fresh task

```bash
npm run sandcastle            # lowest-numbered unblocked `ready` issue
npm run sandcastle -- 7       # a specific issue (#7)
SANDCASTLE_MODEL=claude-opus-4-8 npm run sandcastle -- 7   # escalate the model for a hard brief
```

On failure the driver re-flags the issue `ready` and comments why.

## Iterate on review feedback

```bash
npm run sandcastle:iterate -- 8 "Add a SIGINT test sibling to the SIGTERM one."
npm run sandcastle:iterate -- 8 "Factor out the shared assert helper." --resume
```

- **Default = branch re-entry** (robust): the prior work is committed on
  `agent/issue-<N>`, so a fresh agent reads the code + feedback and adds commits.
  Works on any machine, after any gap.
- **`--resume`** additionally continues the agent's captured Claude session for
  chain-of-thought continuity. Needs a `sessionId` in `state/issue-<N>.json` (written
  by `run.mts` on the same host); if the session can't be found it falls back to
  plain re-entry with a visible warning — never silently.

## Design notes

- **GitHub is host-side only.** The sandbox receives the brief as inert prompt
  arguments and never holds a GitHub token. Selection, locking, PR creation, and
  pushing all run on the host (see `lib.mts`).
- **Branch base is `origin/main`, not local `HEAD`.** Local `main` can drift from
  the remote (e.g. a squash-merge you haven't pulled); the driver always cuts the
  agent branch from `origin/main` so the base is current and deterministic.
- **Branch strategy** is `{ type: "branch" }` so changes reach you as a reviewable
  PR — the agent never touches `main`.
- **State** (`state/issue-<N>.json`) is machine-local and gitignored: it caches the
  PR number and the resumable session id.
- **The lock is not atomic.** One driver at a time; make the `ready`→`in-progress`
  flip atomic before running drivers concurrently.
- **Prompt templates must never contain `` !` ``** (bang immediately before a
  backtick) — sandcastle reads that as a shell command to run in the sandbox.
