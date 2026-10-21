# 0002: keep the tool small and handle realistic failures

## Status

Accepted

## Date

2026-10-21

## Context

t3-worktree-cleanup is a CLI for developers' coding agents. It needs predictable
commands and clear results, while speculative edge cases and race recovery can add
more code than the workflow warrants.

## Decision

Keep the implementation small and simple. Handle realistic failure modes; do not
add speculative edge-case or race-condition hardening. Add recovery behaviour when
the actual workflow justifies it.

For this tool that means:

- No stored plans: `apply` rebuilds the plan and acts only on the paths it is
  given.
- No orphan cleanup: folders that no thread references are left to the user.
- No schema validation layer: a changed T3 schema fails the SQL query with a clear
  error.
- JSON output only; agents format it for users.

The quality bar stays the same: correct behaviour, clear errors, tests and the
[required checks](0003-quality-gates.md).

## Consequences

Changes need a concrete failure or user workflow to justify added complexity.
Some uncommon failures stop with an actionable error. Simplicity does not justify
skipping validation or removing handling for known failures.
