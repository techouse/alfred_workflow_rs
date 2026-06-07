use alfred_workflow_rs::{AutomaticCache, Item, Items};
use pretty_assertions::assert_eq;
use serde_json::json;

fn fixture_items() -> Vec<Item> {
    vec![
        Item::with_arg("First", "first-arg").set_uid("uid-1"),
        Item::with_arg("Second", "second-arg").set_uid("uid-2"),
        Item::with_arg("Third", "third-arg").set_uid("uid-3"),
    ]
}

#[test]
fn items_json_includes_uid_by_default_and_omits_skipknowledge_and_cache()
-> Result<(), Box<dyn std::error::Error>> {
    let items = Items::new(fixture_items());

    assert_eq!(
        serde_json::to_value(items)?,
        json!({
            "items": [
                {
                    "title": "First",
                    "type": "default",
                    "valid": false,
                    "arg": "first-arg",
                    "uid": "uid-1"
                },
                {
                    "title": "Second",
                    "type": "default",
                    "valid": false,
                    "arg": "second-arg",
                    "uid": "uid-2"
                },
                {
                    "title": "Third",
                    "type": "default",
                    "valid": false,
                    "arg": "third-arg",
                    "uid": "uid-3"
                }
            ]
        })
    );

    Ok(())
}

#[test]
fn skipknowledge_true_is_serialized_and_uid_is_preserved() -> Result<(), Box<dyn std::error::Error>>
{
    let items = Items::new(fixture_items()).with_skip_knowledge(true);
    let json = serde_json::to_value(items)?;

    assert_eq!(json["skipknowledge"], json!(true));
    for item in json["items"].as_array().expect("items is an array") {
        assert!(item.get("uid").is_some());
    }

    Ok(())
}

#[test]
fn skipknowledge_false_is_serialized_and_uid_is_preserved() -> Result<(), Box<dyn std::error::Error>>
{
    let items = Items::new(fixture_items()).with_skip_knowledge(false);
    let json = serde_json::to_value(items)?;

    assert_eq!(json["skipknowledge"], json!(false));
    for item in json["items"].as_array().expect("items is an array") {
        assert!(item.get("uid").is_some());
    }

    Ok(())
}

#[test]
fn exact_order_true_removes_uid_from_serialized_items() -> Result<(), Box<dyn std::error::Error>> {
    let items = Items::new(fixture_items()).exact_order(true);
    let json = serde_json::to_value(items)?;

    assert!(json.get("skipknowledge").is_none());
    for item in json["items"].as_array().expect("items is an array") {
        assert!(item.get("uid").is_none());
    }

    Ok(())
}

#[test]
fn exact_order_true_with_skipknowledge_true_removes_uid_and_serializes_skipknowledge()
-> Result<(), Box<dyn std::error::Error>> {
    let items = Items::new(fixture_items())
        .exact_order(true)
        .with_skip_knowledge(true);
    let json = serde_json::to_value(items)?;

    assert_eq!(json["skipknowledge"], json!(true));
    for item in json["items"].as_array().expect("items is an array") {
        assert!(item.get("uid").is_none());
    }

    Ok(())
}

#[test]
fn exact_order_true_with_skipknowledge_false_removes_uid_and_serializes_skipknowledge()
-> Result<(), Box<dyn std::error::Error>> {
    let items = Items::new(fixture_items())
        .exact_order(true)
        .with_skip_knowledge(false);
    let json = serde_json::to_value(items)?;

    assert_eq!(json["skipknowledge"], json!(false));
    for item in json["items"].as_array().expect("items is an array") {
        assert!(item.get("uid").is_none());
    }

    Ok(())
}

#[test]
fn cache_is_serialized_only_when_set() -> Result<(), Box<dyn std::error::Error>> {
    let cache = AutomaticCache::try_with_loose_reload(600, Some(true))?;
    let items = Items::new(fixture_items()).with_cache(cache.clone());

    assert_eq!(
        serde_json::to_value(items)?["cache"],
        serde_json::to_value(cache)?
    );

    Ok(())
}

#[test]
fn items_clone_has_independent_items_vector() {
    let items = Items::new(fixture_items());
    let mut copy = items.clone();
    copy.push(Item::new("Fourth"));

    assert_eq!(items.len(), 3);
    assert_eq!(copy.len(), 4);
}

#[test]
fn items_deserializes_skipknowledge_and_cache() -> Result<(), Box<dyn std::error::Error>> {
    let items: Items = serde_json::from_value(json!({
        "skipknowledge": false,
        "cache": {
            "seconds": 60,
            "loosereload": true
        },
        "items": [
            {
                "title": "First",
                "uid": "uid-1"
            }
        ]
    }))?;

    assert_eq!(items.skip_knowledge(), Some(false));
    assert_eq!(items.cache().expect("cache is present").seconds(), 60);
    assert_eq!(items.items()[0].uid(), Some("uid-1"));

    Ok(())
}
