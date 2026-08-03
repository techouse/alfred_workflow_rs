.DEFAULT_GOAL := help

CARGO ?= cargo
CARGO_MSRV ?= cargo +1.88.0
RUSTDOCFLAGS_DOCS ?= -D warnings --cfg docsrs
PACKAGE_LIST ?= /tmp/alfred-workflow-rs-package-list.txt
PACKAGE_REQUIRED_FILES ?= Cargo.lock Cargo.toml LICENSE README.md examples/auto_update.rs examples/basic.rs examples/caching.rs examples/common/mod.rs src/lib.rs
PACKAGE_EXCLUDED_PATTERN ?= ^(\.codacy\.yml$$|\.github/|\.gitattributes$$|\.gitignore$$|\.history/|\.vscode/|AGENTS\.md$$|about\.hbs$$|about\.toml$$|CODE-OF-CONDUCT\.md$$|docs/|fuzz/|install\.sh$$|Makefile$$|RELEASING\.md$$|SPEC\.md$$|scripts/|src/.*/tests\.rs$$|src/.*/tests/|SECURITY\.md$$|tests/|THIRD-PARTY-LICENSES\.md$$)

.PHONY: help build build-release clean fmt fmt-check clippy test test-all \
	test-doc coverage coverage-html msrv docs docs-missing handoff-check \
	package-list package-contents-check package-check package-check-clean package-check-offline \
	publish-dry-run version-check release-check pre-release ci

help: ## Show available targets
	@awk 'BEGIN {FS = ":.*## "}; /^[a-zA-Z0-9_.-]+:.*## / {printf "%-18s %s\n", $$1, $$2}' $(MAKEFILE_LIST) | sort

build: ## Build the crate
	$(CARGO) build --locked

build-release: ## Build the crate in release mode
	$(CARGO) build --release --locked

clean: ## Remove Cargo build artifacts
	$(CARGO) clean

fmt: ## Format Rust sources
	$(CARGO) fmt

fmt-check: ## Check Rust formatting
	$(CARGO) fmt --check

clippy: ## Run clippy with CI warning policy
	$(CARGO) clippy --all-targets --all-features --locked -- -D warnings

test: ## Run default tests
	$(CARGO) test --locked

test-all: ## Run all-feature tests
	$(CARGO) test --all-features --locked

test-doc: ## Run documentation tests
	$(CARGO) test --doc --locked

coverage: ## Generate an LCOV coverage report (requires cargo-llvm-cov)
	$(CARGO) llvm-cov --all-features --locked --lib --tests --lcov --output-path lcov.info

coverage-html: ## Generate an HTML coverage report (requires cargo-llvm-cov)
	$(CARGO) llvm-cov --all-features --locked --lib --tests --html

msrv: ## Run tests on the crate MSRV (requires toolchain 1.88.0)
	$(CARGO_MSRV) test --locked -- --test-threads=1 --nocapture

docs: ## Build library docs with docs.rs warning settings
	RUSTDOCFLAGS='$(RUSTDOCFLAGS_DOCS)' $(CARGO) doc --locked --no-deps --lib

docs-missing: ## Check public library docs with missing_docs denied
	RUSTFLAGS='-D missing_docs' $(CARGO) check --lib --all-features --locked

handoff-check: ## Check Rust port handoff docs are present and internally anchored
	@test -f README.md
	@test -f LICENSE
	@test -f SPEC.md
	@test -f docs/SPIKE_FINDINGS.md
	@test -f docs/DART_TEST_PARITY.md
	@test -f docs/DART_TO_RUST_API.md
	@test -f docs/DEPENDENCY_REVIEW.md
	@grep -q 'v1.0.0-rc.2' SPEC.md
	@grep -q '^version = "1.0.0-rc.2"$$' Cargo.toml
	@grep -q '^alfred_workflow_rs = "1.0.0-rc.2"$$' README.md
	@grep -q 'alfred_workflow_rs' README.md
	@grep -q 'test/unit/alfred_workflow_test.dart' docs/DART_TEST_PARITY.md
	@grep -q 'test/unit/services/alfred_updater_test.dart' docs/DART_TEST_PARITY.md
	@grep -q 'alfred-workflow' docs/SPIKE_FINDINGS.md
	@grep -q 'Workflow' docs/DART_TO_RUST_API.md

package-list: ## List files included in the published crate package
	$(CARGO) package --locked --list --allow-dirty > $(PACKAGE_LIST)
	@cat $(PACKAGE_LIST)

package-contents-check:
	@for file in $(PACKAGE_REQUIRED_FILES); do \
		grep -q "^$$file$$" $(PACKAGE_LIST) || { echo "package is missing $$file" >&2; exit 1; }; \
	done
	@! grep -E '$(PACKAGE_EXCLUDED_PATTERN)' $(PACKAGE_LIST) || { echo "package contains excluded files" >&2; exit 1; }

package-check: ## Verify crates.io package creation during development
	$(CARGO) package --locked --list --allow-dirty > $(PACKAGE_LIST)
	$(MAKE) package-contents-check
	$(CARGO) package --locked --allow-dirty

package-check-clean: ## Verify clean crates.io package creation
	$(CARGO) package --locked --list > $(PACKAGE_LIST)
	$(MAKE) package-contents-check
	$(CARGO) package --locked

package-check-offline: ## Verify clean crate package creation using only local cache
	$(CARGO) package --locked --list --offline > $(PACKAGE_LIST)
	$(MAKE) package-contents-check
	$(CARGO) package --locked --offline

publish-dry-run: ## Verify crates.io publishability without uploading
	$(CARGO) publish --dry-run --locked

version-check: ## Check release version references agree
	@version="$$(sed -n 's/^version = "\([^"]*\)"$$/\1/p' Cargo.toml | head -n 1)"; \
	if [ -z "$$version" ]; then echo "Could not read version from Cargo.toml" >&2; exit 1; fi; \
	awk -v version="$$version" 'BEGIN { found = 0; in_package = 0 } /^\[\[package\]\]/ { in_package = 0 } /^name = "alfred_workflow_rs"$$/ { in_package = 1 } in_package && $$0 == "version = \"" version "\"" { found = 1 } END { exit !found }' Cargo.lock || { echo "Cargo.lock does not contain alfred_workflow_rs $$version" >&2; exit 1; }; \
	grep -q "^alfred_workflow_rs = \"$$version\"$$" README.md || { echo "README.md install snippet is not using $$version" >&2; exit 1; }; \
	grep -q "v$$version" SPEC.md || { echo "SPEC.md is missing v$$version" >&2; exit 1; }; \
	printf 'version-check: %s\n' "$$version"

release-check: ## Run release readiness audit checks
	$(MAKE) version-check
	$(MAKE) docs
	$(MAKE) docs-missing
	$(MAKE) handoff-check
	$(MAKE) package-list
	$(MAKE) package-check-offline

pre-release: ## Run the full local gate before tagging a release
	$(MAKE) fmt-check
	$(MAKE) clippy
	$(MAKE) test
	$(MAKE) test-all
	$(MAKE) test-doc
	$(MAKE) docs
	$(MAKE) docs-missing
	$(MAKE) msrv
	$(MAKE) version-check
	$(MAKE) handoff-check
	$(MAKE) package-check-clean
	$(MAKE) publish-dry-run
	$(MAKE) build-release

ci: ## Run the main local CI checks
	$(MAKE) fmt-check
	$(MAKE) clippy
	$(MAKE) test
	$(MAKE) test-doc
	$(MAKE) version-check
	$(MAKE) handoff-check
