# 0006: delete uncommitted and untracked files with the worktree

## Status

Accepted

## Date

2026-11-22

## Context

[ADR 0005](0005-own-cleanup-rule.md) skips a worktree with uncommitted or
untracked changes unless `apply --force` is given. Agents leave scratch files,
build output and half-finished edits behind in most worktrees, so many inactive
worktrees stayed on disk. Removing them meant passing `--force`, which also
deletes commits that no remote has.

## Decision

This record supersedes one clause of ADR 0005: "A worktree with uncommitted or
untracked changes [...] is skipped unless `apply --force` is given." Uncommitted
and untracked changes no longer make a worktree `skipped`. A worktree that is inactive for
longer than `--older-than` is a candidate whatever its working tree contains,
and `apply` removes it with `git worktree remove --force`.

Commits that no remote branch contains still make a worktree `skipped`, and only
`apply --force` overrides that. A failed check still can't be forced.

## Consequences

Inactive worktrees are cleaned up without `--force`. Uncommitted, untracked and
ignored files in them, such as `.env`, are lost and can't be restored, and the branch SHA the tool
reports only covers committed work. `--force` now only overrides the
unpushed-commit guard. Locked worktrees still fail to delete.
