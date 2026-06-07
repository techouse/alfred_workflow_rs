use alfred_workflow_rs::AutomaticCache;
use pretty_assertions::assert_eq;
use serde_json::json;

#[test]
fn automatic_cache_serializes_seconds_and_omits_loosereload_when_unset()
-> Result<(), Box<dyn std::error::Error>> {
    let cache = AutomaticCache::try_new(60)?;

    assert_eq!(serde_json::to_value(cache)?, json!({ "seconds": 60 }));

    Ok(())
}

#[test]
fn automatic_cache_serializes_loosereload_when_set() -> Result<(), Box<dyn std::error::Error>> {
    let cache = AutomaticCache::try_with_loose_reload(300, Some(false))?;

    assert_eq!(
        serde_json::to_value(cache)?,
        json!({
            "seconds": 300,
            "loosereload": false
        })
    );

    Ok(())
}

#[test]
fn automatic_cache_accepts_inclusive_seconds_bounds() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(AutomaticCache::try_new(5)?.seconds(), 5);
    assert_eq!(AutomaticCache::try_new(86_400)?.seconds(), 86_400);

    Ok(())
}

#[test]
fn automatic_cache_rejects_seconds_outside_bounds() {
    assert!(AutomaticCache::try_new(4).is_err());
    assert!(AutomaticCache::try_new(86_401).is_err());
}

#[test]
fn automatic_cache_deserialization_validates_seconds() {
    assert!(serde_json::from_value::<AutomaticCache>(json!({ "seconds": 4 })).is_err());
    assert!(serde_json::from_value::<AutomaticCache>(json!({ "seconds": 86_401 })).is_err());
}

#[test]
fn automatic_cache_equality_includes_loosereload() -> Result<(), Box<dyn std::error::Error>> {
    let cache = AutomaticCache::try_with_loose_reload(120, Some(true))?;
    let different = AutomaticCache::try_with_loose_reload(120, Some(false))?;

    assert_ne!(cache, different);

    Ok(())
}
