use alfred_workflow_rs::{
    Action, Icon, IconType, Item, ItemText, ItemType, Modifier, ModifierKey, TypedAction,
};
use pretty_assertions::assert_eq;
use serde_json::json;

#[test]
fn item_serializes_required_title_default_type_and_default_valid()
-> Result<(), Box<dyn std::error::Error>> {
    let item = Item::new("Test Item");

    assert_eq!(
        serde_json::to_value(item)?,
        json!({
            "title": "Test Item",
            "type": "default",
            "valid": false
        })
    );

    Ok(())
}

#[test]
fn item_serializes_optional_fields_with_alfred_wire_keys() -> Result<(), Box<dyn std::error::Error>>
{
    let item = Item::builder("Open file")
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
        .build()?;

    assert_eq!(
        serde_json::to_value(item)?,
        json!({
            "title": "Open file",
            "type": "file:skipcheck",
            "valid": true,
            "subtitle": "A useful file",
            "arg": "~/Documents/report.txt",
            "autocomplete": "report",
            "uid": "report-uid",
            "icon": {
                "path": "icons/report.png",
                "type": "fileicon"
            },
            "text": {
                "copy": "copy text",
                "largetype": "large text"
            },
            "quicklookurl": "https://example.com/report",
            "match": "report document",
            "mods": {
                "cmd+shift": {
                    "arg": "modified-arg",
                    "subtitle": "Modified subtitle",
                    "icon": {
                        "path": "public.png",
                        "type": "filetype"
                    },
                    "valid": false
                }
            }
        })
    );

    Ok(())
}

#[test]
fn modifier_key_sets_serialize_in_canonical_order() -> Result<(), Box<dyn std::error::Error>> {
    let item = Item::builder("With mods")
        .try_modifier(
            [ModifierKey::Ctrl, ModifierKey::Alt],
            Modifier::new().with_subtitle("Alt Ctrl"),
        )?
        .try_modifier(
            [ModifierKey::Shift, ModifierKey::Cmd, ModifierKey::Alt],
            Modifier::new().with_subtitle("Alt Cmd Shift"),
        )?
        .build()?;

    let json = serde_json::to_value(item)?;

    assert_eq!(
        json["mods"],
        json!({
            "alt+cmd+shift": {
                "subtitle": "Alt Cmd Shift",
                "valid": true
            },
            "alt+ctrl": {
                "subtitle": "Alt Ctrl",
                "valid": true
            }
        })
    );

    Ok(())
}

#[test]
fn action_serializes_string_form() -> Result<(), Box<dyn std::error::Error>> {
    let item = Item::with_action("Action item", Action::from("Alfred is Great")).set_valid(true);

    assert_eq!(
        serde_json::to_value(item)?,
        json!({
            "title": "Action item",
            "type": "default",
            "valid": true,
            "action": "Alfred is Great"
        })
    );

    Ok(())
}

#[test]
fn action_serializes_list_form() -> Result<(), Box<dyn std::error::Error>> {
    let item = Item::with_action("Action item", Action::from(vec!["one", "two"])).set_valid(true);

    assert_eq!(
        serde_json::to_value(item)?,
        json!({
            "title": "Action item",
            "type": "default",
            "valid": true,
            "action": ["one", "two"]
        })
    );

    Ok(())
}

#[test]
fn action_serializes_typed_object_form_with_null_fields() -> Result<(), Box<dyn std::error::Error>>
{
    let item = Item::with_action(
        "Typed action",
        TypedAction::text(vec!["one", "two"]).with_url("https://www.alfredapp.com"),
    );

    assert_eq!(
        serde_json::to_value(item)?,
        json!({
            "title": "Typed action",
            "type": "default",
            "valid": false,
            "action": {
                "text": ["one", "two"],
                "url": "https://www.alfredapp.com",
                "file": null,
                "auto": null
            }
        })
    );

    Ok(())
}

#[test]
fn invalid_action_json_types_fail_deserialization() {
    for invalid_action in [
        json!(123),
        json!(true),
        json!(["valid", 123]),
        json!({}),
        json!({ "text": 123 }),
    ] {
        let json = json!({
            "title": "Invalid action",
            "valid": true,
            "action": invalid_action
        });

        assert!(serde_json::from_value::<Item>(json).is_err());
    }
}

#[test]
fn item_builder_rejects_arg_and_action_together() {
    let item = Item::builder("Invalid")
        .arg("argument")
        .action(Action::from("action"))
        .build();

    assert!(item.is_err());
}

#[test]
fn item_deserialization_rejects_arg_and_action_together() {
    let json = json!({
        "title": "Invalid",
        "arg": "argument",
        "action": "action"
    });

    assert!(serde_json::from_value::<Item>(json).is_err());
}

#[test]
fn item_deserialization_canonicalizes_modifier_keys() -> Result<(), Box<dyn std::error::Error>> {
    let item: Item = serde_json::from_value(json!({
        "title": "With mods",
        "mods": {
            "shift+cmd": {
                "subtitle": "Shift Cmd"
            }
        }
    }))?;

    assert_eq!(
        serde_json::to_value(item)?,
        json!({
            "title": "With mods",
            "type": "default",
            "valid": false,
            "mods": {
                "cmd+shift": {
                    "subtitle": "Shift Cmd",
                    "valid": true
                }
            }
        })
    );

    Ok(())
}
