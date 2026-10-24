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

## Domain language

Describe architecture in Domain-Driven Design terms. This applies to
architecture documents, ADRs, issues, PR descriptions and explanations of the
code.

- Use the project's ubiquitous language: the words the domain uses, such as
  `worktree`, `thread` and `cleanup plan`. Use the same word in code, docs and
  conversation, and don't switch between synonyms for one concept.
- Name the parts of the model by their DDD role. An entity keeps its identity as
  its data changes. A value object is defined by its values and is immutable. An
  aggregate is a cluster of objects that changes only through its root.
- Call stateless domain logic that belongs to no single type a domain service,
  and code that runs one use case an application service.
- Call a part of the model with its own language a bounded context. Code that
  reads T3's state database or runs git is an adapter; keep T3's table and
  column names out of the cleanup rule.
- Use these terms to describe the code that exists. Don't add layers, interfaces
  or folders just to match a DDD pattern; keeping the project small still wins.

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
