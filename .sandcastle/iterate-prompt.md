<!-- EDITOR NOTE: never put an exclamation mark immediately before a backtick in
     this file — sandcastle reads that as a shell command to run in the sandbox.
     Write the panic/todo macros without a trailing bang. No shell expansion here. -->

# Iterate on telltaled issue #{{ISSUE_NUMBER}}

You previously implemented this issue; your work is **already committed on the
current branch**. A reviewer has asked for changes. Read `AGENTS.md` for the
workflow, guardrails, and the gate.

## Review feedback to address

{{FEEDBACK}}

## How to work

- Make the **smallest** change that addresses the feedback. Don't refactor or
  change behavior the feedback didn't ask about.
- Stay TDD: add or adjust tests for any behavior change.
- Keep the guardrails (forbid unsafe; no unwrap/expect/panic/todo macros in
  production; complexity / file-length / argument-count thresholds).
- The gate is `just check` (fmt-check, clippy with warnings-as-errors, tests,
  file-length) and it MUST be green before you finish.

## Do NOT touch GitHub or git remotes

Do not run `gh`, push, switch to `main`, or open/edit PRs. The host harness owns
all of that. Just make the change and commit onto the current branch.

## When done

Ensure `just check` is green and your changes are committed, then output exactly:

<promise>COMPLETE</promise>
