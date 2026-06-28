/**
 * Iterate on an open PR from review feedback — the non-manual step 4.
 *
 *   npm run sandcastle:iterate -- 7 "Rename `foo` to `bar` and add a doc comment"
 *   npm run sandcastle:iterate -- 7 "…" --resume   # also continue the agent's session
 *
 * Default (robust): branch re-entry. The prior work is already committed on
 * `agent/issue-<N>`, so a fresh agent reads the code + feedback and adds commits.
 * No session/host-state dependency — works on any machine, after any gap.
 *
 * --resume (enhancement): additionally continue the agent's captured Claude
 * session for prior chain-of-thought. Requires a sessionId in the issue's state
 * file (written by run.mts on the same host). If the session can't be found,
 * this falls back to plain re-entry with an explicit warning — never silently.
 */
import { run } from "@ai-hero/sandcastle";
import {
  agent, sandbox, git, branchFor, findPr, readState, writeState, errMsg,
} from "./lib.mts";

const argv = process.argv.slice(2);
const wantResumeFlag = argv.includes("--resume");
const positional = argv.filter((a) => a !== "--resume");
const issueNumber = Number(positional[0]);
const feedback = positional.slice(1).join(" ").trim();

if (!Number.isInteger(issueNumber) || issueNumber <= 0 || !feedback) {
  throw new Error('usage: npm run sandcastle:iterate -- <issue> "feedback text" [--resume]');
}

const branch = branchFor(issueNumber);
const state = readState(issueNumber);
const pr = state?.pr ?? findPr(branch);
if (pr === undefined) {
  throw new Error(`No open PR found for ${branch}. Iterate is for PRs already opened by a fresh run.`);
}

// Ensure a local `branch` exists at the PR's tip so sandcastle's named-branch
// strategy reuses it (after a gap or clean checkout it may not be local).
git(["fetch", "origin", branch]);
git(["branch", "-f", branch, "FETCH_HEAD"]);

const sessionId = wantResumeFlag ? state?.sessionId : undefined;
if (wantResumeFlag && !sessionId) {
  console.warn("⚠ --resume requested but no captured sessionId in state; using plain re-entry.");
}
console.log(`▶ iterating on #${issueNumber} (PR #${pr}, ${branch})${sessionId ? " [resume]" : ""}`);

const runOpts = {
  name: `iterate-${issueNumber}`,
  agent,
  sandbox,
  promptFile: "./.sandcastle/iterate-prompt.md",
  promptArgs: { ISSUE_NUMBER: String(issueNumber), FEEDBACK: feedback },
  branchStrategy: { type: "branch", branch } as const,
  maxIterations: 1,
};

try {
  let result;
  try {
    result = await run(sessionId ? { ...runOpts, resumeSession: sessionId } : runOpts);
  } catch (err) {
    // Explicit fallback: only when a resume was attempted and the session is the
    // thing that failed. Anything else is a real error — rethrow.
    if (sessionId && /session/i.test(errMsg(err))) {
      console.warn(`⚠ resume failed (${errMsg(err)}); retrying with plain re-entry.`);
      result = await run(runOpts);
    } else {
      throw err;
    }
  }

  if (result.commits.length === 0) {
    console.log("• agent made no new commits — feedback may already be satisfied or too vague. Nothing pushed.");
  } else {
    console.log(`✓ ${result.commits.length} new commit(s). Pushing to PR #${pr}…`);
    git(["push", "origin", branch], { stdio: "inherit" });
    console.log(`✓ PR #${pr} updated.`);
  }
  // Re-persist the (possibly new) sessionId so a second iteration can resume too.
  writeState(issueNumber, { branch, pr, sessionId: result.iterations.at(-1)?.sessionId ?? state?.sessionId });
} catch (err) {
  console.error(`✗ iterate failed: ${errMsg(err)}`);
  process.exitCode = 1;
}
