use alfred_workflow_rs::{FileCache, Item, Items};
use pretty_assertions::assert_eq;
use tempfile::tempdir;

fn first_item() -> Item {
    Item::with_arg("First", "first-arg").set_uid("uid-1")
}

#[test]
fn file_cache_exposes_public_configuration() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let cache = FileCache::<Items>::try_with_config(dir.path(), "test_cache", 5, 60, true)?;

    assert_eq!(cache.path(), dir.path());
    assert_eq!(cache.name(), "test_cache");
    assert_eq!(cache.max_entries(), 5);
    assert_eq!(cache.time_to_live_seconds(), 60);
    assert!(cache.verbose());

    Ok(())
}

#[test]
fn file_cache_instances_with_different_properties_are_not_equal()
-> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let cache = FileCache::<Items>::try_with_config(dir.path(), "test_cache", 5, 60, true)?;
    let different = FileCache::<Items>::try_with_config(dir.path(), "test_cache", 10, 60, true)?;

    assert_ne!(cache, different);

    Ok(())
}

#[test]
fn file_cache_validates_public_configuration() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;

    assert!(FileCache::<Items>::try_with_config(dir.path(), "test_cache", 0, 60, false).is_err());
    assert!(FileCache::<Items>::try_with_config(dir.path(), "test_cache", 5, 4, false).is_err());
    assert!(
        FileCache::<Items>::try_with_config(dir.path(), "test_cache", 5, 86_401, false).is_err()
    );

    Ok(())
}

#[test]
fn cache_keys_are_lower_case_md5_hex() {
    assert_eq!(
        FileCache::<Items>::hash_key("Lorem"),
        "db6ff2ffe2df7b8cfc0d9542bdce27dc"
    );
    assert_eq!(
        FileCache::<Items>::hash_key("ipsum"),
        "e78f5438b48b39bcbdea61b73679449d"
    );
    assert_eq!(
        FileCache::<Items>::hash_key("a"),
        "0cc175b9c0f1b6a831c399e269772661"
    );
}

#[test]
fn file_cache_get_put_and_remove_round_trips_items() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let cache = FileCache::<Items>::try_with_config(dir.path(), "test_cache", 5, 60, false)?;
    let key = FileCache::<Items>::hash_key("query");
    let items = Items::new(vec![first_item()]);

    assert_eq!(cache.get(&key)?, None);
    cache.put(&key, items.clone())?;

    let reopened = FileCache::<Items>::try_with_config(dir.path(), "test_cache", 5, 60, false)?;
    assert_eq!(reopened.get(&key)?, Some(items.clone()));

    reopened.remove(&key)?;
    assert_eq!(cache.get(&key)?, None);

    Ok(())
}
