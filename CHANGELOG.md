## 1.0.2

- Migrated file caching from `cached` 2.x's sled backend to `cached` 4.0.0 with redb; existing disk caches are not reused and are recomputed as needed.
- Raised the minimum supported Rust version to 1.92.
- Updated `plist` to 1.10.1, `ureq` to 3.4.2, and `tempfile` to 3.27.

## 1.0.1

- Fixed file-cache serialization for Script Filter values with omitted optional fields.
- Unreadable file-cache entries are now evicted and treated as cache misses.

## 1.0.0

Initial stable release.
