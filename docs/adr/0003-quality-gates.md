# 0003: gate quality with Clippy, coverage, CRAP and mutation testing

## Status

Accepted

## Date

2026-10-21

## Context

The tool deletes worktrees and branches. Mistakes in the rule or the guards lose
work, so the code that decides what to delete needs tests that would notice a
change in behaviour, not just tests that execute it.

## Decision

`make check` is required before committing. It runs:

- rustfmt and Clippy with warnings as errors.
- Tests, including end-to-end tests against temporary git repositories.
- cargo-deny for advisories, licenses and crate sources.
- cargo-llvm-cov with at least 80 percent line coverage.
- cargo-crap with a CRAP limit of 12 per function, plus its duplicate report.

cargo-mutants checks code changed since `MUTATION_BASE`, defaulting to
`origin/main`, through `make mutants`. Any surviving mutant fails the gate.
`make mutants-all` reports every surviving mutant.

GitHub Actions runs every `make check` gate on pull requests and pushes to `main`,
the changed-code mutation gate on pull requests, and a full mutation run on
manual dispatch.

## Consequences

Changed code must be tested well enough to kill its mutants, which adds test work
for small changes. A mutant that cannot be killed meaningfully needs an
explanation in review. The thresholds are floors: raise them as the code allows,
and do not lower them to pass a change.
