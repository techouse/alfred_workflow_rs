# alfred_workflow_rs

Rust port of the Dart [`alfred_workflow`](https://github.com/techouse/alfred_workflow)
package for building Alfred workflow Script Filter output, reading Alfred user
configuration plists, caching workflow results, and checking GitHub releases for
workflow updates.

The port is guided by Dart package `alfred_workflow` `1.2.4` and its test suite.
Behavioral parity is tracked in [`docs/DART_TEST_PARITY.md`](docs/DART_TEST_PARITY.md).

## Install

```toml
[dependencies]
alfred_workflow_rs = "1.0.0-rc.1"
```

The crate requires Rust `1.88` or newer.

## Basic Usage

```rust
use alfred_workflow_rs::{Item, Result, Workflow};

fn main() -> Result<()> {
    let mut workflow = Workflow::new();
    workflow.add_item(
        Item::with_arg("Search Google", "https://www.google.com/search?q=alfred")
            .set_subtitle("Open search results")
            .set_valid(true),
    )?;

    workflow.write_stdout()
}
```

## Cache Usage

```rust
use alfred_workflow_rs::{FileCache, Item, Items, Result, Workflow};

fn main() -> Result<()> {
    let mut workflow = Workflow::with_file_cache(FileCache::<Items>::new());
    workflow.set_cache_key(Some("query"));

    if workflow.get_items()?.is_empty() {
        workflow.add_item(Item::with_arg("Result", "result-arg"))?;
    }

    workflow.write_stdout()
}
```

## User Configuration

```rust
use alfred_workflow_rs::{Result, Workflow};

fn main() -> Result<()> {
    let workflow = Workflow::new();
    let defaults = workflow.get_user_defaults("info.plist", "prefs.plist")?;

    for (variable, config) in defaults {
        println!("{variable}: {}", config.configuration_type().as_str());
    }

    Ok(())
}
```

## Updater

```rust
use std::time::Duration;

use alfred_workflow_rs::{Result, Updater};

fn main() -> Result<()> {
    let updater = Updater::builder("https://github.com/owner/workflow".parse()?, "1.0.0")?
        .update_interval(Duration::from_secs(7 * 24 * 60 * 60))
        .build()?;

    if updater.update_available()? {
        updater.update()?;
    }

    Ok(())
}
```

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

## Compatibility Notes

- Alfred Script Filter JSON preserves Dart and Alfred wire keys such as
  `quicklookurl`, `largetype`, `skipknowledge`, and `loosereload`.
- `Items::exact_order(true)` removes item `uid` values from serialized output.
- Missing plist files return empty maps in Rust.
- APIs are blocking by default; there is no async feature in the release path.
- The updater opens downloaded `.alfredworkflow` files through an injectable
  opener. Tests never invoke the macOS `open` command.
- The crate is macOS-oriented because Alfred is macOS-only, but the model and
  serialization tests are ordinary Rust tests.

## Development Checks

```sh
make ci
cargo test --examples --locked
make package-check
```

Release-readiness notes:

- [`docs/DART_TO_RUST_API.md`](docs/DART_TO_RUST_API.md): public API mapping and
  intentional Dart divergences.
- [`docs/DEPENDENCY_REVIEW.md`](docs/DEPENDENCY_REVIEW.md): dependency purpose
  and license review.
- [`tests/fixtures/README.md`](tests/fixtures/README.md): fixture provenance.

## License

MIT. See [`LICENSE`](LICENSE).
