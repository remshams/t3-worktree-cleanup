# 0005: apply one inactivity rule to every worktree

## Status

Accepted

## Date

2026-10-21

## Context

T3 Code's built-in storage cleanup keeps a worktree whenever its threads are not a
single subagent tree. A fork, `create_threads`, launching a thread into an
existing worktree or picking an already checked-out branch all make a worktree
shared, and it then stays forever. Archiving or settling the threads doesn't
change that. Fixing this upstream is out of scope.

## Decision

Apply one rule to every worktree with threads: it is a candidate when no thread
using it has had activity for longer than `--older-than` (default 4 days).
Lineage, sharing and settled state are ignored. Activity follows T3's own clock:
thread creation, user messages and run request, start and completion times.

Guards protect work that only exists in the worktree. A worktree with uncommitted
or untracked changes, or with commits that no remote branch contains, is skipped
unless `apply --force` is given. Deleting a worktree also deletes its checked-out
local branch, except the default branch, and reports the branch's commit SHA.

## Consequences

Shared worktrees get cleaned up, and the rule is easy to explain to users.
Worktrees that T3's own cleanup would also remove are covered too; running both is
harmless. Old threads can't recreate their worktree after deletion because the
branch is gone. Commits on deleted branches remain recoverable through the
reported SHA until git garbage-collects them.
