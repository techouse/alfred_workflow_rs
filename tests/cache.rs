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
    let fixtures = [
        ("Lorem", "db6ff2ffe2df7b8cfc0d9542bdce27dc"),
        ("ipsum", "e78f5438b48b39bcbdea61b73679449d"),
        ("dolor", "a98931d104a7fb8f30450547d97e7ca5"),
        ("sit", "87d4eeb7dec7686410748d174c0e0a11"),
        ("amet,", "5a3e3d45a946e52ce224472c5db8b6a6"),
        ("consectetur", "4c480b2170d066b2af6f98af80902ce0"),
        ("adipiscing", "d540f9a8003e11e009342a40200192ea"),
        ("elit.", "0eedc028fc779c2eb13e494a6362135c"),
        ("Duis", "21a253e799186f681f4e520d06395b2f"),
        ("lacinia,", "ec116b1f7146a3cb4d4a92bf1e7cccd9"),
        ("eros", "39eef2554d8407cbfeb017e68c8685e3"),
        ("quis", "bb98d4e9c281b175ea84c517b59308ea"),
        ("consequat", "cb3f4c73e4c498c768489f566045252c"),
        ("condimentum,", "92eba8bdfd8123cc47dbc13f01a1feb2"),
        ("metus", "7259bcad654293e3876bbb6a6febebe1"),
        ("est", "1c52bdae8bad70e82da799843bb4e831"),
        ("scelerisque", "b36b698b5b1246c6d3a5a66aae98a1f5"),
        ("nunc,", "ccf27bc62541abdee63a23c656b0f70b"),
        ("a", "0cc175b9c0f1b6a831c399e269772661"),
        ("Nulla", "0437cb5a0ea1268b32908150f0e26dab"),
        ("semper.", "7edf39407b3ebb85d56777b21c699722"),
    ];

    for (input, expected) in fixtures {
        assert_eq!(FileCache::<Items>::hash_key(input), expected);
    }
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

#[test]
fn file_cache_evicts_least_recently_used_entry_when_max_entries_is_exceeded()
-> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let cache = FileCache::<String>::try_with_config(dir.path(), "test_cache", 2, 60, false)?;

    cache.put("one", "first".to_owned())?;
    cache.put("two", "second".to_owned())?;

    let reopened = FileCache::<String>::try_with_config(dir.path(), "test_cache", 2, 60, false)?;
    assert_eq!(reopened.get("one")?, Some("first".to_owned()));

    reopened.put("three", "third".to_owned())?;

    assert_eq!(cache.get("one")?, Some("first".to_owned()));
    assert_eq!(cache.get("two")?, None);
    assert_eq!(cache.get("three")?, Some("third".to_owned()));

    Ok(())
}
