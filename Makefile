MUTATION_BASE ?= origin/main

.PHONY: check fmt-check lint test deny coverage crap mutants mutants-all

## Run every required gate. Run before committing.
check: fmt-check lint test deny crap

fmt-check:
	cargo fmt --check

lint:
	cargo clippy --all-targets --locked -- -D warnings

test:
	cargo test --locked

deny:
	cargo deny check

## Line coverage of at least 80 percent; writes target/lcov.info.
coverage:
	cargo llvm-cov --locked --lcov --output-path target/lcov.info --fail-under-lines 80

## CRAP score of at most 12 per function, plus a duplicate report.
crap: coverage
	cargo crap --lcov target/lcov.info --threshold 12 --fail-above --duplicates

## Optional locally: no surviving mutants in code changed since MUTATION_BASE.
mutants:
	mkdir -p target
	git diff --src-prefix=a/ --dst-prefix=b/ $(MUTATION_BASE) -- src > target/mutants.diff
	cargo mutants --timeout 60 --in-diff target/mutants.diff

## Report every surviving mutant.
mutants-all:
	cargo mutants --timeout 60
