# 0001: record architecture decisions

## Status

Accepted

## Date

2026-10-21

## Context

The tool deletes data based on decisions about T3's internals, the cleanup rule
and quality gates. Their reasons need a stable home without a lengthy design
process for a small CLI.

## Decision

Record architecture decisions as short Markdown ADRs in `docs/adr/`, using Status,
Date, Context, Decision and Consequences. The [README](README.md) indexes every
record. Accepted records are immutable apart from typo fixes and superseding
status notes. A changed decision needs a new record that links the old one;
partial replacements name the clauses they supersede.

Keep routine implementation choices in commit messages and code review.

## Consequences

Contributors can find a decision's reason and replacement history. Each significant
decision adds a file and an index entry; ordinary changes need neither.
