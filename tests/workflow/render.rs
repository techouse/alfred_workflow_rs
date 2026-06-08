use alfred_workflow_rs::{AutomaticCache, RenderOptions, Workflow};
use pretty_assertions::assert_eq;

use crate::support::{first_item, second_item, third_item};

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
fn write_to_with_applies_render_options() -> Result<(), Box<dyn std::error::Error>> {
    let mut workflow = Workflow::new();
    workflow.add_item(second_item())?;

    let mut output = Vec::new();
    workflow.write_to_with(
        &mut output,
        RenderOptions::new().add_to_beginning(first_item()),
    )?;

    assert_eq!(
        String::from_utf8(output)?,
        r#"{"items":[{"title":"First","type":"default","valid":false,"arg":"first-arg","uid":"uid-1"},{"title":"Second","type":"default","valid":false,"arg":"second-arg","uid":"uid-2"}]}"#
    );

    Ok(())
}

#[test]
fn write_stdout_delegates_to_writer_path() -> Result<(), Box<dyn std::error::Error>> {
    let mut workflow = Workflow::new();
    workflow.add_item(first_item())?;

    workflow.write_stdout()?;
    workflow.write_stdout_with(RenderOptions::new().add_to_end(second_item()))?;

    Ok(())
}
