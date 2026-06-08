use alfred_workflow_rs::{AutomaticCache, Item, RenderOptions, Workflow};
use pretty_assertions::assert_eq;

fn first_item() -> Item {
    Item::with_arg("First", "first-arg").set_uid("uid-1")
}

fn second_item() -> Item {
    Item::with_arg("Second", "second-arg").set_uid("uid-2")
}

fn third_item() -> Item {
    Item::with_arg("Third", "third-arg").set_uid("uid-3")
}

#[test]
fn empty_workflow_returns_empty_items() -> Result<(), Box<dyn std::error::Error>> {
    let workflow = Workflow::new();

    assert!(workflow.get_items()?.is_empty());

    Ok(())
}

#[test]
fn add_item_appends_single_item() -> Result<(), Box<dyn std::error::Error>> {
    let mut workflow = Workflow::new();
    let item = first_item();

    workflow.add_item(item.clone())?;

    assert_eq!(workflow.get_items()?.items(), &[item]);

    Ok(())
}

#[test]
fn add_item_to_beginning_inserts_single_item() -> Result<(), Box<dyn std::error::Error>> {
    let mut workflow = Workflow::new();
    let first = first_item();
    let second = second_item();

    workflow.add_item(second.clone())?;
    workflow.add_item_to_beginning(first.clone())?;

    assert_eq!(workflow.get_items()?.items(), &[first, second]);

    Ok(())
}

#[test]
fn add_items_preserves_order() -> Result<(), Box<dyn std::error::Error>> {
    let mut workflow = Workflow::new();
    let items = vec![first_item(), second_item(), third_item()];

    workflow.add_items(items.clone())?;

    assert_eq!(workflow.get_items()?.items(), items.as_slice());

    Ok(())
}

#[test]
fn clear_items_removes_in_memory_items() -> Result<(), Box<dyn std::error::Error>> {
    let mut workflow = Workflow::new();
    workflow.add_items([first_item(), second_item()])?;

    workflow.clear_items()?;

    assert!(workflow.get_items()?.is_empty());

    Ok(())
}

#[test]
fn to_json_string_renders_default_items() -> Result<(), Box<dyn std::error::Error>> {
    let mut workflow = Workflow::new();
    workflow.add_items([first_item(), second_item()])?;

    assert_eq!(
        workflow.to_json_string()?,
        r#"{"items":[{"title":"First","type":"default","valid":false,"arg":"first-arg","uid":"uid-1"},{"title":"Second","type":"default","valid":false,"arg":"second-arg","uid":"uid-2"}]}"#
    );

    Ok(())
}

#[test]
fn to_json_string_renders_skipknowledge_true_and_false() -> Result<(), Box<dyn std::error::Error>> {
    let mut workflow = Workflow::new();
    workflow.add_item(first_item())?;

    workflow.set_skip_knowledge(Some(true));
    assert_eq!(
        workflow.to_json_string()?,
        r#"{"skipknowledge":true,"items":[{"title":"First","type":"default","valid":false,"arg":"first-arg","uid":"uid-1"}]}"#
    );

    workflow.set_skip_knowledge(Some(false));
    assert_eq!(
        workflow.to_json_string()?,
        r#"{"skipknowledge":false,"items":[{"title":"First","type":"default","valid":false,"arg":"first-arg","uid":"uid-1"}]}"#
    );

    Ok(())
}

#[test]
fn to_json_string_renders_generated_automatic_cache() -> Result<(), Box<dyn std::error::Error>> {
    let mut workflow = Workflow::new();
    workflow.add_item(first_item())?;
    workflow.set_use_automatic_cache(true);

    assert_eq!(
        workflow.to_json_string()?,
        r#"{"cache":{"seconds":60,"loosereload":true},"items":[{"title":"First","type":"default","valid":false,"arg":"first-arg","uid":"uid-1"}]}"#
    );

    Ok(())
}

#[test]
fn to_json_string_renders_custom_automatic_cache() -> Result<(), Box<dyn std::error::Error>> {
    let cache = AutomaticCache::try_with_loose_reload(300, Some(false))?;
    let mut workflow = Workflow::with_automatic_cache(cache);
    workflow.add_item(first_item())?;

    assert_eq!(
        workflow.to_json_string()?,
        r#"{"cache":{"seconds":300,"loosereload":false},"items":[{"title":"First","type":"default","valid":false,"arg":"first-arg","uid":"uid-1"}]}"#
    );

    Ok(())
}

#[test]
fn disable_alfred_smart_result_ordering_removes_uid_from_render()
-> Result<(), Box<dyn std::error::Error>> {
    let mut workflow = Workflow::new();
    workflow.add_items([first_item(), second_item()])?;
    workflow.set_disable_alfred_smart_result_ordering(true);

    assert_eq!(
        workflow.to_json_string()?,
        r#"{"items":[{"title":"First","type":"default","valid":false,"arg":"first-arg"},{"title":"Second","type":"default","valid":false,"arg":"second-arg"}]}"#
    );

    Ok(())
}

#[test]
fn render_options_add_temporary_items_without_mutating_workflow()
-> Result<(), Box<dyn std::error::Error>> {
    let mut workflow = Workflow::new();
    workflow.add_item(second_item())?;

    let json = workflow.to_json_string_with(
        RenderOptions::new()
            .add_to_beginning(first_item())
            .add_to_end(third_item()),
    )?;

    assert_eq!(
        json,
        r#"{"items":[{"title":"First","type":"default","valid":false,"arg":"first-arg","uid":"uid-1"},{"title":"Second","type":"default","valid":false,"arg":"second-arg","uid":"uid-2"},{"title":"Third","type":"default","valid":false,"arg":"third-arg","uid":"uid-3"}]}"#
    );
    assert_eq!(workflow.get_items()?.items(), &[second_item()]);

    Ok(())
}

#[test]
fn write_to_matches_rendered_json() -> Result<(), Box<dyn std::error::Error>> {
    let mut workflow = Workflow::new();
    workflow.add_item(first_item())?;
    let expected = workflow.to_json_string()?;

    let mut output = Vec::new();
    workflow.write_to(&mut output)?;

    assert_eq!(String::from_utf8(output)?, expected);

    Ok(())
}

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
