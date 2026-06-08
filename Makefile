.DEFAULT_GOAL := help

CARGO ?= cargo
RUSTDOCFLAGS_DOCS ?= -D warnings --cfg docsrs
PACKAGE_LIST ?= /tmp/alfred-workflow-rs-package-list.txt

.PHONY: help build build-release clean fmt fmt-check clippy test test-all \
	test-doc coverage coverage-html docs docs-missing handoff-check \
	package-list package-check package-check-clean package-check-offline publish-dry-run \
	release-check pre-release ci

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
	@grep -q 'v1.0.0-rc.1' SPEC.md
	@grep -q '^version = "1.0.0-rc.1"$$' Cargo.toml
	@grep -q '^alfred_workflow_rs = "1.0.0-rc.1"$$' README.md
	@grep -q 'alfred_workflow_rs' README.md
	@grep -q 'test/unit/alfred_workflow_test.dart' docs/DART_TEST_PARITY.md
	@grep -q 'test/unit/services/alfred_updater_test.dart' docs/DART_TEST_PARITY.md
	@grep -q 'alfred-workflow' docs/SPIKE_FINDINGS.md
	@grep -q 'Workflow' docs/DART_TO_RUST_API.md

package-list: ## List files included in the published crate package
	$(CARGO) package --locked --list --allow-dirty > $(PACKAGE_LIST)
	@cat $(PACKAGE_LIST)

package-check: ## Verify crates.io package creation during development
	$(CARGO) package --locked --list --allow-dirty > $(PACKAGE_LIST)
	@grep -q '^README.md$$' $(PACKAGE_LIST)
	@grep -q '^LICENSE$$' $(PACKAGE_LIST)
	@grep -q '^SPEC.md$$' $(PACKAGE_LIST)
	@grep -q '^docs/SPIKE_FINDINGS.md$$' $(PACKAGE_LIST)
	@grep -q '^docs/DART_TEST_PARITY.md$$' $(PACKAGE_LIST)
	@grep -q '^docs/DART_TO_RUST_API.md$$' $(PACKAGE_LIST)
	@grep -q '^docs/DEPENDENCY_REVIEW.md$$' $(PACKAGE_LIST)
	@grep -q '^scripts/regenerate_dart_expected_json.sh$$' $(PACKAGE_LIST)
	@grep -q '^tests/fixtures/README.md$$' $(PACKAGE_LIST)
	@grep -q '^tests/fixtures/info.plist$$' $(PACKAGE_LIST)
	@grep -q '^tests/fixtures/prefs.plist$$' $(PACKAGE_LIST)
	@grep -q '^tests/fixtures/script_filter_full.json$$' $(PACKAGE_LIST)
	@grep -q '^tests/fixtures/script_filter_exact_order.json$$' $(PACKAGE_LIST)
	@grep -q '^tests/fixtures/github_release.json$$' $(PACKAGE_LIST)
	$(CARGO) package --locked --allow-dirty

package-check-clean: ## Verify clean crates.io package creation
	$(CARGO) package --locked --list > $(PACKAGE_LIST)
	@grep -q '^README.md$$' $(PACKAGE_LIST)
	@grep -q '^LICENSE$$' $(PACKAGE_LIST)
	@grep -q '^SPEC.md$$' $(PACKAGE_LIST)
	@grep -q '^docs/SPIKE_FINDINGS.md$$' $(PACKAGE_LIST)
	@grep -q '^docs/DART_TEST_PARITY.md$$' $(PACKAGE_LIST)
	@grep -q '^docs/DART_TO_RUST_API.md$$' $(PACKAGE_LIST)
	@grep -q '^docs/DEPENDENCY_REVIEW.md$$' $(PACKAGE_LIST)
	@grep -q '^scripts/regenerate_dart_expected_json.sh$$' $(PACKAGE_LIST)
	@grep -q '^tests/fixtures/README.md$$' $(PACKAGE_LIST)
	@grep -q '^tests/fixtures/info.plist$$' $(PACKAGE_LIST)
	@grep -q '^tests/fixtures/prefs.plist$$' $(PACKAGE_LIST)
	@grep -q '^tests/fixtures/script_filter_full.json$$' $(PACKAGE_LIST)
	@grep -q '^tests/fixtures/script_filter_exact_order.json$$' $(PACKAGE_LIST)
	@grep -q '^tests/fixtures/github_release.json$$' $(PACKAGE_LIST)
	$(CARGO) package --locked

package-check-offline: ## Verify clean crate package creation using only local cache
	$(CARGO) package --locked --list --offline > $(PACKAGE_LIST)
	@grep -q '^README.md$$' $(PACKAGE_LIST)
	@grep -q '^LICENSE$$' $(PACKAGE_LIST)
	@grep -q '^SPEC.md$$' $(PACKAGE_LIST)
	@grep -q '^docs/SPIKE_FINDINGS.md$$' $(PACKAGE_LIST)
	@grep -q '^docs/DART_TEST_PARITY.md$$' $(PACKAGE_LIST)
	@grep -q '^docs/DART_TO_RUST_API.md$$' $(PACKAGE_LIST)
	@grep -q '^docs/DEPENDENCY_REVIEW.md$$' $(PACKAGE_LIST)
	@grep -q '^scripts/regenerate_dart_expected_json.sh$$' $(PACKAGE_LIST)
	@grep -q '^tests/fixtures/README.md$$' $(PACKAGE_LIST)
	@grep -q '^tests/fixtures/info.plist$$' $(PACKAGE_LIST)
	@grep -q '^tests/fixtures/prefs.plist$$' $(PACKAGE_LIST)
	@grep -q '^tests/fixtures/script_filter_full.json$$' $(PACKAGE_LIST)
	@grep -q '^tests/fixtures/script_filter_exact_order.json$$' $(PACKAGE_LIST)
	@grep -q '^tests/fixtures/github_release.json$$' $(PACKAGE_LIST)
	$(CARGO) package --locked --offline

publish-dry-run: ## Verify crates.io publishability without uploading
	$(CARGO) publish --dry-run --locked

release-check: ## Run release readiness audit checks
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
	$(MAKE) handoff-check
	$(MAKE) package-check-clean
	$(MAKE) publish-dry-run
	$(MAKE) build-release

ci: ## Run the main local CI checks
	$(MAKE) fmt-check
	$(MAKE) clippy
	$(MAKE) test
	$(MAKE) test-doc
	$(MAKE) handoff-check
