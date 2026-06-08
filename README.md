# alfred_workflow_rs

Rust port handoff repo for the Dart `alfred_workflow` package.

This repository is currently documentation-first. The implementation should be
driven by [`SPEC.md`](SPEC.md), with the Dart package and its test suite as the
behavioral source of truth.

## Start Here

- [`SPEC.md`](SPEC.md): authoritative milestone plan from `v0.1` through
  `v1.0.0-rc.1`.
- [`docs/SPIKE_FINDINGS.md`](docs/SPIKE_FINDINGS.md): Rust port decision,
  current crates.io findings, and dependency defaults.
- [`docs/DART_TEST_PARITY.md`](docs/DART_TEST_PARITY.md): mapping from the Dart
  test suite to Rust parity tests.
- [`docs/DART_TO_RUST_API.md`](docs/DART_TO_RUST_API.md): public API translation
  guide for the port.

## Source Of Truth

The source package is:

```text
/Users/klemen/Work/darted/alfred_workflow
```

The current Dart package version inspected for this handoff is `1.2.4`.

The Rust crate name remains `alfred_workflow_rs` for now because
`alfred-workflow` is already taken on crates.io.

## Examples

The Rust examples mirror the Dart package examples and compile with
`cargo test --examples`:

- [`examples/basic.rs`](examples/basic.rs): basic Script Filter JSON output.
- [`examples/caching.rs`](examples/caching.rs): query-keyed file cache usage.
- [`examples/auto_update.rs`](examples/auto_update.rs): updater flow with an
  update action.

Run one with:

```sh
cargo run --example basic -- --query "hello"
```
