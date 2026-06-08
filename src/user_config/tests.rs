use super::*;

use tempfile::tempdir;

fn dictionary(entries: impl IntoIterator<Item = (&'static str, Value)>) -> Dictionary {
    entries
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value))
        .collect()
}

fn text_field_config(variable: &str, default_value: &str) -> Value {
    Value::Dictionary(dictionary([
        ("type", Value::String("textfield".to_owned())),
        ("variable", Value::String(variable.to_owned())),
        ("description", Value::String("description".to_owned())),
        ("label", Value::String("label".to_owned())),
        (
            "config",
            Value::Dictionary(dictionary([
                ("default", Value::String(default_value.to_owned())),
                ("placeholder", Value::String("placeholder".to_owned())),
                ("required", Value::Boolean(true)),
                ("trim", Value::Boolean(false)),
            ])),
        ),
    ]))
}

#[test]
fn parser_helpers_accept_valid_values_and_report_schema_errors() -> Result<()> {
    let values = dictionary([
        ("string", Value::String("value".to_owned())),
        ("bool", Value::Boolean(true)),
        ("integer", Value::Integer(42.into())),
        ("unsigned", Value::Integer(42_u64.into())),
        ("overflow", Value::Integer(u64::MAX.into())),
        (
            "dict",
            Value::Dictionary(dictionary([("inner", Value::String("value".to_owned()))])),
        ),
        (
            "pairs",
            Value::Array(vec![Value::Array(vec![
                Value::String("Label".to_owned()),
                Value::String("Stored".to_owned()),
            ])]),
        ),
    ]);

    assert_eq!(required_string(&values, "string")?, "value");
    assert_eq!(optional_string(&values, "string"), Some("value".to_owned()));
    assert_eq!(optional_string(&values, "missing"), None);
    assert!(required_bool(&values, "bool")?);
    assert_eq!(required_integer(&values, "integer")?, 42);
    assert_eq!(required_integer(&values, "unsigned")?, 42);
    assert_eq!(optional_integer(&values, "missing"), None);
    assert_eq!(optional_integer(&values, "overflow"), None);
    assert_eq!(
        required_dictionary_key(&values, "dict")?.get("inner"),
        Some(&Value::String("value".to_owned()))
    );
    assert_eq!(
        required_select_pairs(&values)?,
        vec![SelectPair {
            label: "Label".to_owned(),
            value: "Stored".to_owned(),
        }]
    );
    assert_eq!(plist_integer(&Value::Integer(7_u64.into())), Some(7));
    assert_eq!(plist_integer(&Value::Integer(u64::MAX.into())), None);
    assert_eq!(
        plist_integer(&Value::String("not an integer".to_owned())),
        None
    );

    let default_string = dictionary([("default", Value::String("default".to_owned()))]);
    let default_bool = dictionary([("default", Value::Boolean(false))]);
    let default_integer = dictionary([("defaultvalue", Value::Integer(9.into()))]);
    assert_eq!(
        default_value(&default_string),
        Some(&Value::String("default".to_owned()))
    );
    assert_eq!(required_default_string(&default_string)?, "default");
    assert!(!required_default_bool(&default_bool)?);
    assert_eq!(required_default_integer(&default_integer)?, 9);
    assert!(default_value(&Dictionary::new()).is_none());

    assert!(required_dictionary(&Value::String("not a dict".to_owned()), "context").is_err());
    assert!(required_dictionary_key(&values, "missing").is_err());
    assert!(required_string(&values, "missing").is_err());
    assert!(required_bool(&values, "missing").is_err());
    assert!(required_integer(&values, "missing").is_err());
    assert!(required_default_string(&Dictionary::new()).is_err());
    assert!(required_default_bool(&Dictionary::new()).is_err());
    assert!(required_default_integer(&Dictionary::new()).is_err());
    assert!(required_select_pairs(&Dictionary::new()).is_err());
    assert!(
        required_select_pairs(&dictionary([(
            "pairs",
            Value::Array(vec![Value::String("not a pair".to_owned())]),
        )]))
        .is_err()
    );
    assert!(
        required_select_pairs(&dictionary([(
            "pairs",
            Value::Array(vec![Value::Array(vec![
                Value::Boolean(true),
                Value::String("value".to_owned()),
            ])]),
        )]))
        .is_err()
    );
    assert!(
        required_select_pairs(&dictionary([(
            "pairs",
            Value::Array(vec![Value::Array(vec![
                Value::String("label".to_owned()),
                Value::Boolean(true),
            ])]),
        )]))
        .is_err()
    );

    let string_error = schema_error(String::from("owned"));
    let str_error = schema_error("borrowed");
    assert!(matches!(string_error, Error::UserConfiguration(_)));
    assert!(matches!(str_error, Error::UserConfiguration(_)));

    Ok(())
}

#[test]
fn public_parsers_and_workflow_methods_parse_unit_test_fixture() -> Result<()> {
    let directory = tempdir()?;
    let info_path = directory.path().join("info.plist");
    let prefs_path = directory.path().join("prefs.plist");
    let info = Value::Dictionary(dictionary([(
        "userconfigurationconfig",
        Value::Array(vec![text_field_config("text_variable", "default text")]),
    )]));
    let prefs = Value::Dictionary(dictionary([(
        "text_variable",
        Value::String("configured text".to_owned()),
    )]));
    info.to_file_xml(&info_path)
        .map_err(|error| Error::UserConfiguration(error.to_string()))?;
    prefs
        .to_file_xml(&prefs_path)
        .map_err(|error| Error::UserConfiguration(error.to_string()))?;

    let workflow = Workflow::new();
    let defaults = get_defaults(&info_path)?;
    let workflow_defaults = workflow.get_defaults(&info_path)?;
    let preferences = get_user_preferences(&prefs_path)?;
    let workflow_preferences = workflow.get_user_preferences(&prefs_path)?;
    let merged = get_user_defaults(&info_path, &prefs_path)?;
    let workflow_merged = workflow.get_user_defaults(&info_path, &prefs_path)?;

    assert_eq!(defaults.len(), 1);
    assert_eq!(workflow_defaults, defaults);
    assert_eq!(preferences, workflow_preferences);
    assert_eq!(merged, workflow_merged);

    let config = &workflow_merged["text_variable"];
    assert_eq!(
        UserConfigurationType::from_wire("textfield"),
        Some(UserConfigurationType::TextField)
    );
    assert_eq!(UserConfigurationType::from_wire("unknown"), None);
    assert_eq!(
        config.configuration_type(),
        UserConfigurationType::TextField
    );
    assert_eq!(config.variable(), "text_variable");
    assert_eq!(config.description(), Some("description"));
    assert_eq!(config.label(), Some("label"));

    assert!(matches!(
        config,
        UserConfiguration::TextField(text_field)
            if text_field.config.default_value == "default text"
                && text_field.config.value == "configured text"
    ));

    Ok(())
}
