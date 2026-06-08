# Test Fixtures

Fixture provenance for the v0.8 parity audit:

- `info.plist` and `prefs.plist` are copied from
  `/Users/klemen/Work/darted/alfred_workflow/test/fixtures/data` as inspected
  from Dart package `alfred_workflow` `1.2.4`.
- `script_filter_full.json` and `script_filter_exact_order.json` are
  deterministic JSON snapshots generated from stable Dart model construction.
  Regenerate them with `scripts/regenerate_dart_expected_json.sh` when the Dart
  source package changes.
- `github_release.json` is a deterministic GitHub release payload matching the
  updater model shape used by the Dart updater fixtures.
