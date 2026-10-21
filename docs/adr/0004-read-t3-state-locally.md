# 0004: read T3 state locally and change it only through git

## Status

Accepted

## Date

2026-10-21

## Context

The tool needs every thread's worktree path and last activity, across all
projects, on several machines. T3 Code offers three sources:

- The state database `<T3 home>/userdata/statev2.sqlite`, in WAL mode.
- The server's HTTP and WebSocket API, which needs a bearer token per machine and
  doesn't return archived threads over HTTP.
- The T3 MCP tools, which don't return worktree paths or activity times and use a
  different authentication model.

## Decision

Write the tool in Rust as a single binary with SQLite built in. Read the state
database with a read-only connection, in one query. Install the tool on each
machine with a T3 server and reach other machines over SSH. The T3 home defaults
to `$T3CODE_HOME`, then `~/.t3`, and can be set with `--base-dir`.

Never write to T3's database. Change the filesystem only through `git worktree
remove` and `git branch -D`, run from the project's root.

## Consequences

The tool needs no token, works while the T3 server is down and sees archived and
deleted threads. It depends on T3's internal schema, which changes often; a
changed schema fails the query with an error. `store.rs` is the only module that
knows the schema. The tool can't tell T3 that a worktree is gone, so the threads
keep their worktree path and T3 can't recreate it once the branch is deleted.
