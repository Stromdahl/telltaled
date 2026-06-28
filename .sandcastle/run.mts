/**
 * Fresh run: pick an unblocked `ready` sub-task, lock it `in-progress`, implement
 * the brief in a Docker sandbox on `agent/issue-<N>`, push, open a PR. Captures the
 * agent's session id so `iterate --resume` can continue it later.
 *
 *   npm run sandcastle           # lowest-numbered unblocked `ready` issue
 *   npm run sandcastle -- 7      # a specific ready issue
 *
 * See iterate.mts for the review-feedback loop. Lock flip is not atomic — one
 * driver at a time.
 */
import { run } from "@ai-hero/sandcastle";
import {
  agent, sandbox, BASE, READY, LOCK, BLOCKED,
  gh, git, branchFor, findPr, writeState, errMsg, type Issue,
} from "./lib.mts";

function pickIssue(): Issue {
  const explicit = process.argv[2];
  const list: Issue[] = JSON.parse(
    gh(["issue", "list", "--label", READY, "--state", "open",
        "--json", "number,title,body,labels", "--limit", "100"]),
  );
  const ready = list
    .filter((i) => !i.labels.some((l) => l.name === BLOCKED || l.name === LOCK))
    .sort((a, b) => a.number - b.number);

  if (explicit !== undefined) {
    const n = Number(explicit);
    const found = ready.find((i) => i.number === n);
    if (!found) throw new Error(`#${explicit} is not an unblocked \`${READY}\` issue.`);
    return found;
  }
  if (ready.length === 0) throw new Error(`No unblocked \`${READY}\` issues to work on.`);
  return ready[0];
}

const issue = pickIssue();
const branch = branchFor(issue.number);
console.log(`▶ #${issue.number}: ${issue.title}  →  ${branch}`);

// Cut the agent branch from the up-to-date integration base, NOT local HEAD.
// Local `main` can drift from `origin/${BASE}` (e.g. a teammate merged a PR you
// haven't pulled), and sandcastle's named-branch strategy reuses whatever commit
// `branch` already points at. Forcing `branch` to `origin/${BASE}` here makes the
// base deterministic and current, and overwrites any stale branch from a prior run.
git(["fetch", "origin", BASE]);
git(["branch", "-f", branch, "FETCH_HEAD"]);
console.log(`  (based ${branch} on origin/${BASE})`);

// Grab the lock on the host before launching the agent.
gh(["issue", "edit", String(issue.number), "--remove-label", READY, "--add-label", LOCK]);

try {
  const result = await run({
    name: `issue-${issue.number}`,
    agent,
    sandbox,
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

  if (result.commits.length === 0) throw new Error("agent finished without committing anything");

  console.log(`✓ ${result.commits.length} commit(s) on ${result.branch}. Pushing + opening PR…`);
  git(["push", "-u", "origin", result.branch], { stdio: "inherit" });
  gh(["pr", "create",
      "--base", BASE,
      "--head", result.branch,
      "--title", `${issue.title} (#${issue.number})`,
      "--body", `Automated implementation of #${issue.number} by a sandcastle-orchestrated agent.\n\nCloses #${issue.number}.`]);

  const pr = findPr(result.branch);
  writeState(issue.number, { branch, pr, sessionId: result.iterations.at(-1)?.sessionId });
  console.log(`✓ PR #${pr ?? "?"} opened. Iterate on review feedback with: npm run sandcastle:iterate -- ${issue.number} "…"`);
} catch (err) {
  const msg = errMsg(err);
  console.error(`✗ run failed: ${msg}`);
  // Release the lock so the issue is grabbable again.
  gh(["issue", "edit", String(issue.number), "--remove-label", LOCK, "--add-label", READY]);
  gh(["issue", "comment", String(issue.number),
      "--body", `Automated sandcastle run failed: ${msg}\n\nRe-flagged \`${READY}\`.`]);
  process.exitCode = 1;
}
