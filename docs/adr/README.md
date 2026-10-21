# Architecture decision records

This directory holds t3-worktree-cleanup's architecture decisions. Each record is
one Markdown file named `NNNN-kebab-title.md`, numbered in decision order. Use the
[template](template.md), with five sections: Status, Date, Context, Decision and
Consequences. No front matter is needed.

## When to write a record

Record decisions that change module responsibilities or dependencies, the CLI
contract, the cleanup rule, how T3 data is read, or the quality gates contributors
must follow. Explain the constraint, choice and tradeoff. Test names, routine
dependency bumps and implementation details within an existing boundary belong in
commits and code review.

## Rules

- A record starts as Proposed. Once approved, its status becomes Accepted.
- Accepted records are immutable. Fixing a typo is fine; changing the decision
  requires a new record.
- If a whole decision changes, retain the old content and update only its Status
  to `Superseded by ADR NNNN`, linking the replacement.
- If only part changes, use `Partially superseded by ADR NNNN; all other decisions
  remain accepted.` The replacement must name the clauses it supersedes and link
  the old record.
- Add each record to the index and keep its status current.

## Index

| Record | Title | Status |
| --- | --- | --- |
| [0001](0001-record-architecture-decisions.md) | Record architecture decisions | Accepted |
| [0002](0002-keep-the-tool-small.md) | Keep the tool small and handle realistic failures | Accepted |
| [0003](0003-quality-gates.md) | Gate quality with Clippy, coverage, CRAP and mutation testing | Accepted |
| [0004](0004-read-t3-state-locally.md) | Read T3 state locally and change it only through git | Accepted |
| [0005](0005-own-cleanup-rule.md) | Apply one inactivity rule to every worktree | Accepted |
