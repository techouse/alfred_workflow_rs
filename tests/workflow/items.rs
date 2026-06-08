use alfred_workflow_rs::Workflow;
use pretty_assertions::assert_eq;

use crate::support::{first_item, second_item, third_item};

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
