# Sandcastle orchestration for telltaled

Runs AI coding agents in an isolated Docker sandbox against telltaled's
**`ready`-flag** task workflow: pick an unblocked `ready` sub-task issue → lock it
`in-progress` → implement the brief on a branch → push + open a PR for review.

Built on [`@ai-hero/sandcastle`](https://github.com/mattpocock/sandcastle). The Rust
repo stays the source of truth; this is just the agent harness.

## Files

| File | Purpose |
|------|---------|
| `main.mts` | The driver: issue selection, host-side lock/PR, the `run()` call. |
| `prompt.md` | The agent's brief — injects the issue body and points at `AGENTS.md` + `just check`. |
| `Dockerfile` | Sandbox image: Claude Code CLI + Rust toolchain + `just`. No `gh` (GitHub stays host-side). |
| `.env.example` | Template for agent auth (`CLAUDE_CODE_OAUTH_TOKEN` / `ANTHROPIC_API_KEY`). |

## One-time setup

```bash
npm install                                  # installs sandcastle + tsx
cp .sandcastle/.env.example .sandcastle/.env # then fill in your token
npm run sandcastle:build-image               # builds the Docker image (Rust toolchain — slow first time)
gh auth status                               # the driver uses host gh for labels + PRs
```

## Run

```bash
npm run sandcastle            # work the lowest-numbered unblocked `ready` issue
npm run sandcastle -- 7       # work a specific issue (#7)
SANDCASTLE_MODEL=claude-opus-4-8 npm run sandcastle -- 7   # escalate the model for a hard brief
```

The driver locks the issue (`ready` → `in-progress`), runs the agent on branch
`agent/issue-<N>`, then pushes and opens a PR. On failure it re-flags the issue
`ready` and comments why.

## Design notes

- **GitHub is host-side only.** The sandbox receives the brief as inert prompt
  arguments and never holds a GitHub token. Selection, locking, and PR creation
  all run on the host in `main.mts`.
- **Branch strategy** is `{ type: "branch" }` so changes land on `agent/issue-<N>`
  and reach you as a reviewable PR — local `main` is never touched by the agent.
- **The lock is not atomic.** Safe for one driver; make it atomic before running
  drivers concurrently against the same repo.
