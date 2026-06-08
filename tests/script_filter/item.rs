use alfred_workflow_rs::{
    Action, ActionText, Icon, IconType, Item, ItemText, ItemType, Modifier, ModifierKey,
    TypedAction,
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

#[test]
fn action_conversions_and_typed_action_accessors_cover_supported_forms()
-> Result<(), Box<dyn std::error::Error>> {
    let typed = TypedAction::try_new(
        Some(ActionText::from(String::from("copy text"))),
        Some(String::from("https://example.com")),
        Some(String::from("/tmp/file")),
        Some(String::from("auto text")),
    )?;

    assert_eq!(typed.text_value(), Some(&ActionText::from("copy text")));
    assert_eq!(typed.url_value(), Some("https://example.com"));
    assert_eq!(typed.file_value(), Some("/tmp/file"));
    assert_eq!(typed.auto_value(), Some("auto text"));

    assert_eq!(
        TypedAction::url("https://example.com").url_value(),
        Some("https://example.com")
    );
    assert_eq!(
        TypedAction::file("/tmp/file").file_value(),
        Some("/tmp/file")
    );
    assert_eq!(
        TypedAction::auto("auto text").auto_value(),
        Some("auto text")
    );
    assert_eq!(
        TypedAction::url("https://example.com")
            .with_text(vec![String::from("one"), String::from("two")])
            .with_file("/tmp/file")
            .with_auto("auto text")
            .text_value(),
        Some(&ActionText::from(vec![
            String::from("one"),
            String::from("two")
        ]))
    );

    assert_eq!(
        Action::from(String::from("owned")),
        Action::String(String::from("owned"))
    );
    assert_eq!(
        Action::from(vec![Action::from("one"), Action::from("two")]),
        Action::List(vec![Action::from("one"), Action::from("two")])
    );
    assert_eq!(
        Action::from(vec![String::from("one"), String::from("two")]),
        Action::List(vec![Action::from("one"), Action::from("two")])
    );
    assert_eq!(Action::from(typed.clone()), Action::Typed(typed));
    assert!(TypedAction::try_new(None, None, None, None).is_err());

    Ok(())
}

#[test]
fn action_text_conversions_serialization_and_deserialization_cover_supported_forms()
-> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        ActionText::from(String::from("owned")),
        ActionText::String(String::from("owned"))
    );
    assert_eq!(
        ActionText::from("borrowed"),
        ActionText::String(String::from("borrowed"))
    );
    assert_eq!(
        ActionText::from(vec![String::from("one"), String::from("two")]),
        ActionText::List(vec![String::from("one"), String::from("two")])
    );
    assert_eq!(
        ActionText::from(vec!["one", "two"]),
        ActionText::List(vec![String::from("one"), String::from("two")])
    );
    assert_eq!(
        serde_json::to_value(ActionText::from("copy"))?,
        json!("copy")
    );
    assert_eq!(
        serde_json::to_value(ActionText::from(vec!["one", "two"]))?,
        json!(["one", "two"])
    );
    assert_eq!(
        serde_json::from_value::<ActionText>(json!("copy"))?,
        ActionText::from("copy")
    );
    assert_eq!(
        serde_json::from_value::<ActionText>(json!(["one", "two"]))?,
        ActionText::from(vec!["one", "two"])
    );
    assert!(serde_json::from_value::<ActionText>(json!(["one", 2])).is_err());
    assert!(serde_json::from_value::<ActionText>(json!(true)).is_err());

    Ok(())
}

#[test]
fn item_related_accessors_return_configured_values() -> Result<(), Box<dyn std::error::Error>> {
    let icon = Icon::new("icons/document.png").with_type(IconType::FileIcon);
    let file_type_icon = Icon::new("public.png").with_type(IconType::FileType);
    let text = ItemText::new("copy value").with_large_type("large value");
    let modifier = Modifier::new()
        .with_arg("mod-arg")
        .with_subtitle("Modified subtitle")
        .with_icon(file_type_icon.clone())
        .with_valid(false);

    assert_eq!(ItemType::Default.as_str(), "default");
    assert_eq!(ItemType::File.as_str(), "file");
    assert_eq!(ItemType::FileSkipcheck.as_str(), "file:skipcheck");
    assert_eq!(IconType::FileIcon.as_str(), "fileicon");
    assert_eq!(IconType::FileType.as_str(), "filetype");
    assert_eq!(icon.path(), "icons/document.png");
    assert_eq!(icon.icon_type(), Some(IconType::FileIcon));
    assert_eq!(text.copy(), "copy value");
    assert_eq!(text.large_type(), Some("large value"));
    assert_eq!(modifier.arg(), Some("mod-arg"));
    assert_eq!(modifier.subtitle(), Some("Modified subtitle"));
    assert_eq!(modifier.icon(), Some(&file_type_icon));
    assert!(!modifier.valid());

    for (key, wire) in [
        (ModifierKey::Cmd, "cmd"),
        (ModifierKey::Ctrl, "ctrl"),
        (ModifierKey::Alt, "alt"),
        (ModifierKey::Shift, "shift"),
        (ModifierKey::Fn, "fn"),
    ] {
        assert_eq!(key.as_str(), wire);
        assert_eq!(wire.parse::<ModifierKey>()?, key);
        assert_eq!(serde_json::to_value(key)?, json!(wire));
    }
    assert!("unknown".parse::<ModifierKey>().is_err());

    let item = Item::new("Configured")
        .set_item_type(ItemType::File)
        .set_valid(true)
        .set_subtitle("Subtitle")
        .set_arg("argument")
        .set_autocomplete("auto")
        .set_uid("uid")
        .set_icon(icon.clone())
        .set_text(text.clone())
        .set_quick_look_url("https://example.com/preview")
        .set_match_text("match terms")
        .try_set_modifier([ModifierKey::Cmd, ModifierKey::Cmd], modifier.clone())?;

    assert_eq!(item.title(), "Configured");
    assert_eq!(item.item_type(), ItemType::File);
    assert!(item.valid());
    assert_eq!(item.subtitle(), Some("Subtitle"));
    assert_eq!(item.arg(), Some("argument"));
    assert_eq!(item.autocomplete(), Some("auto"));
    assert_eq!(item.uid(), Some("uid"));
    assert_eq!(item.icon(), Some(&icon));
    assert_eq!(item.text(), Some(&text));
    assert_eq!(item.quick_look_url(), Some("https://example.com/preview"));
    assert_eq!(item.match_text(), Some("match terms"));
    assert_eq!(
        item.modifiers()
            .and_then(|mods| mods.get("cmd"))
            .expect("cmd modifier present"),
        &modifier
    );
    assert_eq!(item.action(), None);

    let action_item = item.clone().set_action("action value");
    assert_eq!(action_item.arg(), None);
    assert_eq!(action_item.action(), Some(&Action::from("action value")));

    let arg_item = action_item.set_arg("new arg");
    assert_eq!(arg_item.arg(), Some("new arg"));
    assert_eq!(arg_item.action(), None);

    Ok(())
}

#[test]
fn item_builder_sets_every_field_and_modifier_validation_rejects_empty_key_sets()
-> Result<(), Box<dyn std::error::Error>> {
    let icon = Icon::new("icons/document.png");
    let text = ItemText::new("copy");
    let modifier = Modifier::new().with_subtitle("Modified");

    let item = Item::builder("Built")
        .item_type(ItemType::FileSkipcheck)
        .valid(true)
        .subtitle("Subtitle")
        .arg("argument")
        .autocomplete("auto")
        .uid("uid")
        .icon(icon.clone())
        .text(text.clone())
        .quick_look_url("https://example.com/preview")
        .match_text("match terms")
        .try_modifier([ModifierKey::Shift], modifier.clone())?
        .build()?;

    assert_eq!(item.item_type(), ItemType::FileSkipcheck);
    assert_eq!(item.subtitle(), Some("Subtitle"));
    assert_eq!(item.arg(), Some("argument"));
    assert_eq!(item.autocomplete(), Some("auto"));
    assert_eq!(item.uid(), Some("uid"));
    assert_eq!(item.icon(), Some(&icon));
    assert_eq!(item.text(), Some(&text));
    assert_eq!(item.quick_look_url(), Some("https://example.com/preview"));
    assert_eq!(item.match_text(), Some("match terms"));
    assert_eq!(
        item.modifiers()
            .and_then(|mods| mods.get("shift"))
            .expect("shift modifier present"),
        &modifier
    );

    assert!(
        Item::builder("Invalid")
            .try_modifier([], Modifier::new())
            .is_err()
    );
    assert!(
        Item::new("Invalid")
            .try_set_modifier([], Modifier::new())
            .is_err()
    );

    Ok(())
}

#[test]
fn item_deserialization_rejects_unknown_wire_values_and_empty_modifier_keys() {
    assert!(serde_json::from_value::<ItemType>(json!("not-a-type")).is_err());
    assert!(serde_json::from_value::<IconType>(json!("not-an-icon-type")).is_err());
    assert!(serde_json::from_value::<ModifierKey>(json!("not-a-modifier")).is_err());
    assert!(
        serde_json::from_value::<Item>(json!({
            "title": "Invalid modifier",
            "mods": {
                "": {
                    "subtitle": "empty key"
                }
            }
        }))
        .is_err()
    );
}
