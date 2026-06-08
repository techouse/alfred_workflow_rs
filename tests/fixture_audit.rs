use alfred_workflow_rs::{
    Action, AutomaticCache, GithubRelease, Icon, IconType, Item, ItemText, ItemType, Items,
    Modifier, ModifierKey, TypedAction, Updater,
};
use pretty_assertions::assert_eq;
use semver::Version;

fn full_script_filter_fixture() -> alfred_workflow_rs::Result<Items> {
    Ok(Items::new(vec![
        Item::builder("Open file")
            .item_type(ItemType::FileSkipcheck)
            .valid(true)
            .subtitle("A useful file")
            .arg("~/Documents/report.txt")
            .autocomplete("report")
            .uid("report-uid")
            .icon(Icon::new("icons/report.png").with_type(IconType::FileIcon))
            .text(ItemText::new("copy text").with_large_type("large text"))
            .quick_look_url("https://example.com/report")
            .match_text("report document")
            .try_modifier(
                [ModifierKey::Shift, ModifierKey::Cmd],
                Modifier::new()
                    .with_arg("modified-arg")
                    .with_subtitle("Modified subtitle")
                    .with_icon(Icon::new("public.png").with_type(IconType::FileType))
                    .with_valid(false),
            )?
            .build()?,
        Item::with_action(
            "Typed action",
            Action::from(
                TypedAction::text(vec!["one", "two"]).with_url("https://www.alfredapp.com"),
            ),
        ),
    ])
    .with_skip_knowledge(true)
    .with_cache(AutomaticCache::try_with_loose_reload(600, Some(true))?))
}

fn exact_order_fixture() -> Items {
    Items::new(vec![
        Item::with_arg("First", "first-arg").set_uid("uid-1"),
        Item::with_arg("Second", "second-arg").set_uid("uid-2"),
        Item::with_arg("Third", "third-arg").set_uid("uid-3"),
    ])
    .exact_order(true)
}

#[test]
fn representative_script_filter_json_matches_dart_golden() -> Result<(), Box<dyn std::error::Error>>
{
    assert_eq!(
        serde_json::to_string(&full_script_filter_fixture()?)?,
        include_str!("fixtures/script_filter_full.json").trim()
    );

    Ok(())
}

#[test]
fn exact_order_script_filter_json_matches_dart_golden() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        serde_json::to_string(&exact_order_fixture())?,
        include_str!("fixtures/script_filter_exact_order.json").trim()
    );

    Ok(())
}

#[test]
fn github_release_fixture_matches_updater_model() -> Result<(), Box<dyn std::error::Error>> {
    let release: GithubRelease =
        serde_json::from_str(include_str!("fixtures/github_release.json"))?;
    let updater = Updater::new("https://github.com/example/workflow".parse()?, "1.0.0")?;

    assert_eq!(release.tag_name, Version::parse("2.0.1")?);
    assert_eq!(
        updater
            .find_alfred_workflow_asset(&release)
            .map(|asset| asset.id),
        Some(2)
    );

    Ok(())
}
