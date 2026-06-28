/**
 * Shared helpers for the telltaled sandcastle harness (run.mts + iterate.mts).
 * All GitHub/git interaction lives host-side here; the sandbox never holds a token.
 */
import { claudeCode } from "@ai-hero/sandcastle";
import { docker } from "@ai-hero/sandcastle/sandboxes/docker";
import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";

export const REPO = process.cwd();
export const READY = "ready";
export const LOCK = "in-progress";
export const BLOCKED = "blocked";
export const BASE = "main";
/** Default to Sonnet (the briefs' suggested tier); override per run via env. */
export const MODEL = process.env.SANDCASTLE_MODEL ?? "claude-sonnet-4-6";

/** The agent + sandbox are the same across fresh runs and iterations. */
export const agent = claudeCode(MODEL);
export const sandbox = docker();

export interface Issue {
  number: number;
  title: string;
  body: string;
  labels: { name: string }[];
}

/** Per-issue harness state, persisted across invocations (gitignored). */
export interface IssueState {
  branch: string;
  pr?: number;
  /** Claude session id from the last run — enables `iterate --resume`. */
  sessionId?: string;
}

export function gh(args: string[], opts: { stdio?: "inherit" } = {}): string {
  return execFileSync("gh", args, { cwd: REPO, encoding: "utf8", ...opts });
}

export function git(args: string[], opts: { stdio?: "inherit" } = {}): string {
  return execFileSync("git", args, { cwd: REPO, encoding: "utf8", ...opts });
}

export function branchFor(issueNumber: number): string {
  return `agent/issue-${issueNumber}`;
}

function stateFile(issueNumber: number): string {
  return join(REPO, ".sandcastle", "state", `issue-${issueNumber}.json`);
}

export function readState(issueNumber: number): IssueState | undefined {
  try {
    return JSON.parse(readFileSync(stateFile(issueNumber), "utf8")) as IssueState;
  } catch {
    return undefined;
  }
}

export function writeState(issueNumber: number, state: IssueState): void {
  const file = stateFile(issueNumber);
  mkdirSync(dirname(file), { recursive: true });
  writeFileSync(file, JSON.stringify(state, null, 2) + "\n");
}

/** Open PR number whose head is `branch`, or undefined. */
export function findPr(branch: string): number | undefined {
  const out = gh([
    "pr", "list", "--head", branch, "--state", "open",
    "--json", "number", "--jq", ".[0].number // empty",
  ]).trim();
  return out ? Number(out) : undefined;
}

export function errMsg(err: unknown): string {
  return err instanceof Error ? err.message : String(err);
}
