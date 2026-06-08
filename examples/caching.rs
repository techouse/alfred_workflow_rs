mod common;

use alfred_workflow_rs::{Item, Result, Workflow};

fn main() -> Result<()> {
    let mut workflow = Workflow::new();
    let exit_code = match populate_workflow(&mut workflow) {
        Ok(()) => 0,
        Err(error) => {
            workflow.add_item(Item::new(error.to_string()))?;
            1
        }
    };

    workflow.write_stdout()?;
    if exit_code != 0 {
        std::process::exit(exit_code);
    }

    Ok(())
}

fn populate_workflow(workflow: &mut Workflow) -> Result<()> {
    let query = common::query_from_env();

    if query.is_empty() {
        workflow.add_item(common::placeholder_item())?;
    } else {
        workflow.set_cache_key(Some(query.as_str()));
        if workflow.get_items()?.is_empty() {
            workflow.add_item(common::google_item(&query)?)?;
        }
    }

    Ok(())
}
