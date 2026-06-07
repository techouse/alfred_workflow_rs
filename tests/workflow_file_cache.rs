use alfred_workflow_rs::{FileCache, Item, Items, RenderOptions, Workflow};
use pretty_assertions::assert_eq;
use tempfile::tempdir;

fn first_item() -> Item {
    Item::with_arg("First", "first-arg").set_uid("uid-1")
}

fn second_item() -> Item {
    Item::with_arg("Second", "second-arg").set_uid("uid-2")
}

fn third_item() -> Item {
    Item::with_arg("Third", "third-arg").set_uid("uid-3")
}

fn workflow_with_cache(path: &std::path::Path, key: &str) -> Workflow {
    let mut workflow = Workflow::with_file_cache(FileCache::<Items>::with_path(path));
    workflow.set_cache_key(Some(key));
    workflow
}

#[test]
fn file_cache_get_items_without_adding_anything_is_empty() -> Result<(), Box<dyn std::error::Error>>
{
    let dir = tempdir()?;
    let workflow = workflow_with_cache(dir.path(), "query");

    assert!(workflow.get_items()?.is_empty());

    Ok(())
}

#[test]
fn file_cache_add_item_adds_single_item() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let mut workflow = workflow_with_cache(dir.path(), "query");
    let item = first_item();

    workflow.add_item(item.clone())?;

    assert_eq!(workflow.get_items()?.items(), &[item]);

    Ok(())
}

#[test]
fn file_cache_add_items_adds_multiple_items() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let mut workflow = workflow_with_cache(dir.path(), "query");
    let items = vec![first_item(), second_item(), third_item()];

    workflow.add_items(items.clone())?;

    assert_eq!(workflow.get_items()?.items(), items.as_slice());

    Ok(())
}

#[test]
fn file_cache_add_item_appends_to_cached_items() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let mut workflow = workflow_with_cache(dir.path(), "query");

    workflow.add_items([first_item(), second_item()])?;
    workflow.add_item(third_item())?;

    let items = workflow.get_items()?;
    assert_eq!(items.items(), &[first_item(), second_item(), third_item()]);

    Ok(())
}

#[test]
fn file_cache_add_item_to_beginning_prepends_to_cached_items()
-> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let mut workflow = workflow_with_cache(dir.path(), "query");

    workflow.add_items([second_item(), third_item()])?;
    workflow.add_item_to_beginning(first_item())?;

    let items = workflow.get_items()?;
    assert_eq!(items.items(), &[first_item(), second_item(), third_item()]);

    Ok(())
}

#[test]
fn file_cache_clear_items_removes_cached_value() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let mut workflow = workflow_with_cache(dir.path(), "query");

    workflow.add_items([first_item(), second_item()])?;
    let cache_key = workflow.cache_key_hash().expect("cache key is set");

    workflow.clear_items()?;

    assert!(workflow.get_items()?.is_empty());
    assert_eq!(workflow.file_cache().get(&cache_key)?, None);

    Ok(())
}

#[test]
fn file_cache_to_json_string_renders_cached_items() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let mut writer = workflow_with_cache(dir.path(), "query");
    writer.add_items([first_item(), second_item()])?;

    let reader = workflow_with_cache(dir.path(), "query");

    assert_eq!(
        reader.to_json_string()?,
        r#"{"items":[{"title":"First","type":"default","valid":false,"arg":"first-arg","uid":"uid-1"},{"title":"Second","type":"default","valid":false,"arg":"second-arg","uid":"uid-2"}]}"#
    );

    Ok(())
}

#[test]
fn file_cache_render_options_are_temporary_and_not_cached() -> Result<(), Box<dyn std::error::Error>>
{
    let dir = tempdir()?;
    let mut workflow = workflow_with_cache(dir.path(), "query");
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
fn file_cache_settings_follow_workflow_ttl_and_max_entries()
-> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let mut workflow = Workflow::with_file_cache(FileCache::<Items>::with_path(dir.path()));

    assert_eq!(
        workflow.file_cache().time_to_live_seconds(),
        Workflow::DEFAULT_CACHE_TIME_TO_LIVE
    );
    assert_eq!(
        workflow.file_cache().max_entries(),
        Workflow::DEFAULT_MAX_CACHE_ENTRIES
    );

    workflow.set_cache_time_to_live(Some(300));
    assert_eq!(workflow.file_cache().time_to_live_seconds(), 300);

    workflow.set_cache_time_to_live(Some(4));
    assert_eq!(
        workflow.file_cache().time_to_live_seconds(),
        Workflow::DEFAULT_CACHE_TIME_TO_LIVE
    );

    workflow.set_max_cache_entries(Some(5));
    assert_eq!(workflow.file_cache().max_entries(), 5);

    workflow.set_max_cache_entries(Some(0));
    assert_eq!(
        workflow.file_cache().max_entries(),
        Workflow::DEFAULT_MAX_CACHE_ENTRIES
    );

    Ok(())
}
