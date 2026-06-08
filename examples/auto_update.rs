mod common;

use std::time::Duration;

use alfred_workflow_rs::{Icon, Item, RenderOptions, Result, Updater, Workflow};

const UPDATE_ARG: &str = "update:workflow";

fn main() -> Result<()> {
    let updater = Updater::builder("https://github.com/your/repo".parse()?, "1.0.0")?
        .update_interval(Duration::from_secs(7 * 24 * 60 * 60))
        .build()?;

    if update_requested() {
        return updater.update();
    }

    let mut workflow = Workflow::new();
    let (exit_code, options) = match populate_workflow(&mut workflow, &updater) {
        Ok(options) => (0, options),
        Err(error) => {
            workflow.add_item(Item::new(error.to_string()))?;
            (1, RenderOptions::new())
        }
    };

    workflow.write_stdout_with(options)?;
    if exit_code != 0 {
        std::process::exit(exit_code);
    }

    Ok(())
}

fn populate_workflow(workflow: &mut Workflow, updater: &Updater) -> Result<RenderOptions> {
    let query = common::query_from_env();

    if query.is_empty() {
        workflow.add_item(common::placeholder_item())?;
    } else {
        workflow.set_cache_key(Some(query.as_str()));
        if workflow.get_items()?.is_empty() {
            workflow.add_item(common::google_item(&query)?)?;
        }
    }

    if updater.update_available()? {
        Ok(RenderOptions::new().add_to_beginning(update_item()))
    } else {
        Ok(RenderOptions::new())
    }
}

fn update_requested() -> bool {
    std::env::args()
        .skip(1)
        .any(|arg| arg == "-u" || arg == "--update" || arg == UPDATE_ARG)
}

fn update_item() -> Item {
    Item::with_arg("Auto-Update available!", UPDATE_ARG)
        .set_subtitle("Press <enter> to auto-update to a new version of this workflow.")
        .set_match_text(
            "Auto-Update available! Press <enter> to auto-update to a new version of this workflow.",
        )
        .set_icon(Icon::new("alfredhatcog.png"))
        .set_valid(true)
}
