# t3-worktree-cleanup

Deletes [T3 Code](https://github.com/pingdotgg/t3code) worktrees whose threads
have had no activity for a given period (default 4 days), together with their
local branches.

T3's built-in storage cleanup keeps a worktree forever when several threads
share it, for example after a fork or when a second thread is started in the
same worktree. This tool applies one rule to every worktree: if none of the
threads using it has had activity for longer than the limit, it can go.

## Install

```sh
cargo install --locked --git https://github.com/remshams/t3-worktree-cleanup
```

Install it on every machine that runs a T3 server. To clean another machine,
run it there, for example over SSH.

## Usage

```sh
t3-worktree-cleanup plan [--older-than 4d] [--project <title|id|root>]... [--base-dir ~/.t3]
t3-worktree-cleanup apply [same options] [--force] <worktree path>...
```

`plan` prints JSON with candidates and skipped worktrees, their threads, last
activity, size and branch. `apply` re-evaluates the same rule, deletes only the
given paths that still qualify, and prints one result per path. It exits 1 if
any path failed and 2 if the tool could not run.

- **Activity** uses T3's own clock: thread creation, the latest user message and
  the latest run request, start or completion. Renaming, settling or opening a
  thread doesn't count.
- **Skipped**: worktrees with uncommitted or untracked changes, or with commits
  that no remote branch contains. `apply --force` deletes them anyway, but not
  worktrees whose git checks failed.
- **Branches**: the worktree's checked-out branch is deleted locally, except the
  repository's default branch. `apply` reports the branch's last commit;
  `git branch <name> <sha>` restores it until git garbage-collects the commits.
- **After deletion** T3 can't recreate the worktree for the old threads, because
  the branch is gone. Start a new thread instead.

The base directory defaults to `$T3CODE_HOME`, then `~/.t3`. The tool only
reads `<base>/userdata/statev2.sqlite`.

## Agent skill

[`skills/t3-worktree-cleanup/SKILL.md`](skills/t3-worktree-cleanup/SKILL.md)
teaches coding agents to preview, ask for confirmation and then apply. Link it
into your agent's skill directory:

```sh
ln -s "$PWD/skills/t3-worktree-cleanup" ~/.claude/skills/t3-worktree-cleanup
```

## Development

See [AGENTS.md](AGENTS.md). `make check` runs every required gate.
