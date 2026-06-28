/**
 * Sandcastle driver for telltaled's "ready-flag" task workflow.
 *
 * Picks an unblocked `ready` sub-task issue, grabs the `in-progress` lock on the
 * host, runs a Claude Code agent in a Docker sandbox to implement the brief on a
 * dedicated branch, then pushes the branch and opens a PR. All GitHub state lives
 * host-side here; the sandbox sees only the brief (as inert prompt args) + code.
 *
 *   npm run sandcastle              # pick the lowest-numbered ready issue
 *   npm run sandcastle -- 7         # work a specific ready issue (#7)
 *
 * Prerequisites:
 *   - `npm run sandcastle:build-image` once (and after Dockerfile changes).
 *   - `gh auth status` green on the host.
 *   - Claude auth: copy .sandcastle/.env.example -> .sandcastle/.env and fill it,
 *     or export CLAUDE_CODE_OAUTH_TOKEN / ANTHROPIC_API_KEY in your shell.
 *
 * NOTE: the ready->in-progress label flip is NOT atomic — fine for a single
 * driver, but two concurrent drivers could both grab the same issue. Make the
 * lock atomic before running drivers in parallel.
 */
import { run, claudeCode } from "@ai-hero/sandcastle";
import { docker } from "@ai-hero/sandcastle/sandboxes/docker";
import { execFileSync } from "node:child_process";

const REPO = process.cwd();
const READY = "ready";
const LOCK = "in-progress";
const BLOCKED = "blocked";
const BASE = "main";
// Default to Sonnet (the brief's suggested tier; escalate per-issue if needed).
const MODEL = process.env.SANDCASTLE_MODEL ?? "claude-sonnet-4-6";

interface Issue {
  number: number;
  title: string;
  body: string;
  labels: { name: string }[];
}

function gh(args: string[]): string {
  return execFileSync("gh", args, { cwd: REPO, encoding: "utf8" });
}

function pickIssue(): Issue {
  const explicit = process.argv[2];
  const list: Issue[] = JSON.parse(
    gh([
      "issue", "list",
      "--label", READY,
      "--state", "open",
      "--json", "number,title,body,labels",
      "--limit", "100",
    ]),
  );
  const ready = list
    .filter((i) => !i.labels.some((l) => l.name === BLOCKED || l.name === LOCK))
    .sort((a, b) => a.number - b.number);

  if (explicit !== undefined) {
    const n = Number(explicit);
    const found = ready.find((i) => i.number === n);
    if (!found) {
      throw new Error(`#${explicit} is not an unblocked \`${READY}\` issue.`);
    }
    return found;
  }
  if (ready.length === 0) {
    throw new Error(`No unblocked \`${READY}\` issues to work on.`);
  }
  return ready[0];
}

const issue = pickIssue();
const branch = `agent/issue-${issue.number}`;
console.log(`▶ #${issue.number}: ${issue.title}  →  ${branch}`);

// Grab the lock on the host before launching the agent.
gh(["issue", "edit", String(issue.number), "--remove-label", READY, "--add-label", LOCK]);

try {
  const result = await run({
    name: `issue-${issue.number}`,
    agent: claudeCode(MODEL),
    sandbox: docker(),
    promptFile: "./.sandcastle/prompt.md",
    promptArgs: {
      ISSUE_NUMBER: String(issue.number),
      ISSUE_TITLE: issue.title,
      // Backticks, !`...` and {{...}} inside the body are inert: sandcastle only
      // expands shell blocks present in the template, not in substituted values.
      ISSUE_BODY: issue.body,
    },
    branchStrategy: { type: "branch", branch },
    maxIterations: 1,
  });

  if (result.commits.length === 0) {
    throw new Error("agent finished without committing anything");
  }

  console.log(`✓ ${result.commits.length} commit(s) on ${result.branch}. Pushing + opening PR…`);
  execFileSync("git", ["push", "-u", "origin", result.branch], { cwd: REPO, stdio: "inherit" });
  gh([
    "pr", "create",
    "--base", BASE,
    "--head", result.branch,
    "--title", `${issue.title} (#${issue.number})`,
    "--body", `Automated implementation of #${issue.number} by a sandcastle-orchestrated agent.\n\nCloses #${issue.number}.`,
  ]);
  console.log("✓ PR opened. Issue left `in-progress` pending review.");
} catch (err) {
  const msg = err instanceof Error ? err.message : String(err);
  console.error(`✗ run failed: ${msg}`);
  // Release the lock so the issue is grabbable again.
  gh(["issue", "edit", String(issue.number), "--remove-label", LOCK, "--add-label", READY]);
  gh(["issue", "comment", String(issue.number),
      "--body", `Automated sandcastle run failed: ${msg}\n\nRe-flagged \`${READY}\`.`]);
  process.exitCode = 1;
}
