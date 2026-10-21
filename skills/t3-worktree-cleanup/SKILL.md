---
name: t3-worktree-cleanup
description: Find and delete T3 Code worktrees whose threads have been inactive for days, including worktrees shared by several threads. Use when the user asks to clean up, prune or free disk space from T3 worktrees, or asks which T3 worktrees can be deleted.
---

# T3 worktree cleanup

`t3-worktree-cleanup` deletes T3 Code worktrees and their local branches when no
thread using the worktree has had activity for a given period. Always preview,
show the user what will be deleted and get explicit confirmation before `apply`.

## 1. Preview

```sh
t3-worktree-cleanup plan [--older-than 4d] [--project <title|id|root>]...
```

Add `--base-dir <dir>` when T3 doesn't use `$T3CODE_HOME` or `~/.t3`. For another
machine, run the same command there, for example `ssh <host> t3-worktree-cleanup plan`.

The output is JSON:

- `candidates`, `candidateBytes`, `skipped`, `recent`: counts and the disk space
  candidates use. Recent worktrees are only counted.
- `worktrees[]`: `path`, `project`, `status` (`candidate` or `skipped`),
  `reasons`, `lastActivityAt`, `sizeBytes`, `branch`, `deleteBranch` and
  `threads[]` with `id`, `title`, `settledAt` and `lastActivityAt`.

## 2. Show the user

Present a table with one row per worktree: project, worktree folder name,
branch, last activity, size, number of threads and thread titles. List skipped
worktrees separately with their reasons. State the total size of the candidates.

Tell the user that:

- the worktree's local branch is deleted too, unless `deleteBranch` is false;
- the listed threads can't recreate their worktree afterwards, so continuing
  that work needs a new thread;
- skipped worktrees have uncommitted changes or unpushed commits, which `--force`
  would delete.

Ask which worktrees to delete. Never pass `--force` unless the user explicitly
asks to delete a skipped worktree.

## 3. Apply

Pass exactly the confirmed paths, with the same `--older-than`, `--project` and
`--base-dir` options as the preview:

```sh
t3-worktree-cleanup apply [options] <path>...
t3-worktree-cleanup apply [options] --force <path>...   # only confirmed skipped ones
```

`apply` re-checks each path and prints one result per path with `status`
(`deleted`, `skipped` or `failed`), `reasons` and `branch` (`name`, `sha`,
`deleted`, `note`). Exit code 1 means at least one path failed; 2 means the tool
could not run.

Report what was deleted and any failures. Include each deleted branch with its
SHA so the user can restore it with `git -C <project root> branch <name> <sha>`
until git garbage-collects the commits.
