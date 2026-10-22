# t3-worktree-cleanup

A Rust CLI for developers' coding agents that deletes T3 Code worktrees whose
threads have been inactive for a while, including worktrees shared by several
threads, which T3's own cleanup keeps. It reads T3's state database read-only
and runs git for the deletions.

Run `cargo run -- --help`. `plan` previews, `apply <path>...` deletes; both
print JSON. The [agent skill](skills/t3-worktree-cleanup/SKILL.md) describes
the preview, confirm and apply flow agents must follow.

## Architecture documentation

Start with [the architecture document](docs/architecture.md) for module ownership,
dependency rules, runtime flow and tests. Update it in the same change when
modules, dependencies or ownership change. The [ADR index](docs/adr/README.md)
records decisions and explains when to add or supersede a record.

## Development

Keep the project small and simple. Handle realistic failure modes only; avoid
speculative edge-case and race-condition hardening.

Run `make check` before committing. It covers formatting, Clippy, tests,
cargo-deny, line coverage and the CRAP gate. Mutation testing is optional
locally: `make mutants` checks code changed since `origin/main`, configurable
through `MUTATION_BASE`; `make mutants-all` reports every surviving mutant.

Tests must never touch the real `~/.t3`. Use the temporary T3 home and git
repositories in `tests/cli.rs`.

## GitHub issues

Follow the [GitHub issue guide](docs/github-issues.md) when creating or updating an
issue. Apply the `unslop` skill to the title and body before publishing.

## Pull request descriptions

Follow the [pull request description guide](docs/pull-request-descriptions.md)
when creating or updating a PR description. Apply the `unslop` skill before
publishing it.

## CodeRabbit

- CodeRabbit does not review pull requests automatically.
- When the work on a PR is finished and pushed, request a review once with
  `gh pr comment <n> --body "@coderabbitai review"`. Do not request one after
  every push; batch fixes and request one review per round.
- The main branch ruleset requires CodeRabbit's approval. After the last fix,
  request a final review and wait for the approval before merging.
- If CodeRabbit replies that the rate limit is exceeded, wait the stated time,
  then request the review again.

## Commits

- Use Conventional Commits: `<type>(<scope>): <description>`, e.g. `feat(Plan): ...`.
  Valid types: `feat`, `fix`, `refactor`, `chore`, `build`, `docs`, `perf`, `test`,
  `style`, `ci`, `revert`.
- Include the affected component or module name as the scope, in PascalCase.
- For e2e changes, prefix the component/module scope with `e2e-`, e.g.
  `test(e2e-Apply): ...`.
- Unless instructed otherwise, commit changes in meaningful implementation chunks.
- Never mix files from different components or modules in one commit. Each commit
  must have a single component/module scope. Split changes across modules into
  separate commits. Repository moves and codebase formatting may each use one commit.
