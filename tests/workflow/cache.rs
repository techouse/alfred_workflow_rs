use alfred_workflow_rs::{AutomaticCache, FileCache, Items, Workflow};
use pretty_assertions::assert_eq;
use tempfile::tempdir;

#[test]
fn automatic_cache_and_cache_key_flags_are_mutually_exclusive()
-> Result<(), Box<dyn std::error::Error>> {
    let mut workflow = Workflow::new();

    workflow.set_cache_key(Some("cache-key"));
    assert_eq!(workflow.cache_key(), Some("cache-key"));

    workflow.set_use_automatic_cache(true);
    assert_eq!(workflow.cache_key(), None);
    assert!(workflow.use_automatic_cache());

    workflow.set_cache_key(Some("cache-key"));
    assert_eq!(workflow.cache_key(), Some("cache-key"));
    assert!(!workflow.use_automatic_cache());
    assert_eq!(workflow.automatic_cache(), None);

    Ok(())
}

#[test]
fn cache_key_suppresses_constructor_automatic_cache() -> Result<(), Box<dyn std::error::Error>> {
    let cache = AutomaticCache::try_with_loose_reload(300, Some(false))?;
    let mut workflow = Workflow::with_automatic_cache(cache.clone());

    assert_eq!(workflow.automatic_cache(), Some(cache));

    workflow.set_cache_key(Some("cache-key"));

    assert_eq!(workflow.cache_key(), Some("cache-key"));
    assert!(!workflow.use_automatic_cache());
    assert_eq!(workflow.automatic_cache(), None);

    Ok(())
}

#[test]
fn workflow_builder_configures_render_and_cache_flags() -> Result<(), Box<dyn std::error::Error>> {
    let workflow = Workflow::builder()
        .disable_alfred_smart_result_ordering(true)
        .skip_knowledge(Some(false))
        .cache_time_to_live(Some(300))
        .use_automatic_cache(true)
        .build();

    assert!(workflow.disable_alfred_smart_result_ordering());
    assert_eq!(workflow.skip_knowledge(), Some(false));
    assert_eq!(workflow.cache_time_to_live(), Some(300));
    assert_eq!(
        workflow.automatic_cache().map(|cache| cache.seconds()),
        Some(300)
    );

    Ok(())
}

#[test]
fn workflow_default_and_file_cache_accessors_cover_cache_state()
-> Result<(), Box<dyn std::error::Error>> {
    let default_workflow = Workflow::default();
    assert!(default_workflow.get_items()?.is_empty());

    let dir = tempdir()?;
    let file_cache =
        FileCache::<Items>::try_with_config(dir.path(), "workflow_cache", 2, 120, true)?;
    let mut workflow = Workflow::with_file_cache(file_cache.clone());

    assert_eq!(workflow.file_cache(), &file_cache);
    workflow.set_cache_key(Some("query"));
    assert_eq!(workflow.cache_key(), Some("query"));
    assert_eq!(
        workflow.cache_key_hash(),
        Some(FileCache::<Items>::hash_key("query"))
    );

    workflow.clear_cache_key();
    assert_eq!(workflow.cache_key(), None);
    assert_eq!(workflow.cache_key_hash(), None);

    let replacement =
        FileCache::<Items>::try_with_config(dir.path(), "replacement_cache", 3, 300, false)?;
    workflow.set_file_cache(replacement.clone());
    assert_eq!(workflow.file_cache(), &replacement);

    Ok(())
}

#[test]
fn workflow_builder_configures_file_cache_and_disables_automatic_cache_when_requested()
-> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let file_cache =
        FileCache::<Items>::try_with_config(dir.path(), "builder_cache", 4, 240, false)?;
    let workflow = Workflow::builder()
        .file_cache(file_cache.clone())
        .use_automatic_cache(true)
        .max_cache_entries(Some(4))
        .cache_time_to_live(None)
        .cache_key(Some("query"))
        .build();

    assert_eq!(workflow.file_cache().path(), file_cache.path());
    assert_eq!(workflow.file_cache().name(), file_cache.name());
    assert_eq!(workflow.file_cache().max_entries(), 4);
    assert_eq!(
        workflow.file_cache().time_to_live_seconds(),
        Workflow::DEFAULT_CACHE_TIME_TO_LIVE
    );
    assert_eq!(workflow.file_cache().verbose(), file_cache.verbose());
    assert_eq!(workflow.cache_key(), Some("query"));
    assert!(!workflow.use_automatic_cache());
    assert_eq!(workflow.max_cache_entries(), Some(4));
    assert_eq!(
        workflow.effective_cache_time_to_live(),
        Workflow::DEFAULT_CACHE_TIME_TO_LIVE
    );

    Ok(())
}

#[test]
fn workflow_builder_cache_key_suppresses_automatic_cache() -> Result<(), Box<dyn std::error::Error>>
{
    let cache = AutomaticCache::try_with_loose_reload(300, Some(false))?;
    let workflow = Workflow::builder()
        .automatic_cache(cache)
        .cache_key(Some("query"))
        .build();

    assert_eq!(workflow.cache_key(), Some("query"));
    assert_eq!(workflow.automatic_cache(), None);

    Ok(())
}

#[test]
fn cache_time_to_live_validates_range_and_affects_generated_cache()
-> Result<(), Box<dyn std::error::Error>> {
    let mut workflow = Workflow::new();

    assert_eq!(workflow.cache_time_to_live(), None);
    assert_eq!(
        workflow.effective_cache_time_to_live(),
        Workflow::DEFAULT_CACHE_TIME_TO_LIVE
    );
    assert_eq!(workflow.automatic_cache(), None);

    workflow.set_cache_time_to_live(Some(300));
    workflow.set_use_automatic_cache(true);
    assert_eq!(workflow.cache_time_to_live(), Some(300));
    assert_eq!(
        workflow.automatic_cache().map(|cache| cache.seconds()),
        Some(300)
    );

    workflow.set_cache_time_to_live(Some(4));
    assert_eq!(workflow.cache_time_to_live(), None);
    assert_eq!(
        workflow.automatic_cache().map(|cache| cache.seconds()),
        Some(Workflow::DEFAULT_CACHE_TIME_TO_LIVE)
    );

    workflow.set_cache_time_to_live(Some(86_401));
    assert_eq!(workflow.cache_time_to_live(), None);
    assert_eq!(
        workflow.automatic_cache().map(|cache| cache.seconds()),
        Some(Workflow::DEFAULT_CACHE_TIME_TO_LIVE)
    );

    Ok(())
}

#[test]
fn max_cache_entries_validates_value_and_disables_automatic_cache()
-> Result<(), Box<dyn std::error::Error>> {
    let mut workflow = Workflow::new();

    assert_eq!(workflow.max_cache_entries(), None);
    assert_eq!(
        workflow.effective_max_cache_entries(),
        Workflow::DEFAULT_MAX_CACHE_ENTRIES
    );

    workflow.set_use_automatic_cache(true);
    workflow.set_max_cache_entries(Some(5));
    assert_eq!(workflow.max_cache_entries(), Some(5));
    assert_eq!(workflow.effective_max_cache_entries(), 5);
    assert!(!workflow.use_automatic_cache());

    workflow.set_max_cache_entries(Some(0));
    assert_eq!(workflow.max_cache_entries(), None);
    assert_eq!(
        workflow.effective_max_cache_entries(),
        Workflow::DEFAULT_MAX_CACHE_ENTRIES
    );

    Ok(())
}
