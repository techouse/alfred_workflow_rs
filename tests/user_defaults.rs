use std::path::Path;

use alfred_workflow_rs::{
    UserConfiguration, UserConfigurationType, UserPreferences, Workflow,
    user_config::{get_defaults, get_user_preferences},
};
use plist::{Dictionary, Value};
use pretty_assertions::assert_eq;
use tempfile::tempdir;

const INFO_PLIST: &str = "tests/fixtures/info.plist";
const PREFS_PLIST: &str = "tests/fixtures/prefs.plist";

fn write_plist(path: &Path, body: &str) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::write(
        path,
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "https://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
{body}
</plist>"#
        ),
    )?;
    Ok(())
}

#[test]
fn missing_user_default_plists_return_empty_maps() -> Result<(), Box<dyn std::error::Error>> {
    let workflow = Workflow::new();

    assert!(
        workflow
            .get_defaults("tests/fixtures/missing-info.plist")?
            .is_empty()
    );
    assert!(
        workflow
            .get_user_preferences("tests/fixtures/missing-prefs.plist")?
            .is_empty()
    );
    assert!(
        workflow
            .get_user_defaults(
                "tests/fixtures/missing-info.plist",
                "tests/fixtures/missing-prefs.plist",
            )?
            .is_empty()
    );

    Ok(())
}

#[test]
fn get_defaults_parses_all_user_configuration_variants() -> Result<(), Box<dyn std::error::Error>> {
    let defaults = Workflow::new().get_defaults(INFO_PLIST)?;

    assert_eq!(defaults.len(), 6);
    assert_eq!(
        defaults["textfield_variable"].configuration_type(),
        UserConfigurationType::TextField
    );
    assert_eq!(
        defaults["number_slider_variable"].configuration_type(),
        UserConfigurationType::Slider
    );

    let UserConfiguration::TextField(text_field) = &defaults["textfield_variable"] else {
        panic!("textfield config expected");
    };
    assert_eq!(text_field.variable, "textfield_variable");
    assert_eq!(text_field.config.default_value, "textfield default");
    assert_eq!(text_field.config.value, text_field.config.default_value);
    assert_eq!(
        text_field.config.placeholder.as_deref(),
        Some("textfield placeholder")
    );
    assert!(text_field.config.required);
    assert!(!text_field.config.trim);
    assert_eq!(
        text_field.description.as_deref(),
        Some("textfield description")
    );
    assert_eq!(text_field.label.as_deref(), Some("textfield label"));

    let UserConfiguration::Select(select) = &defaults["popupbutton_variable"] else {
        panic!("select config expected");
    };
    assert_eq!(select.config.default_value, "baz value");
    assert_eq!(
        select
            .config
            .pairs
            .iter()
            .map(|pair| (pair.label.as_str(), pair.value.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("foo label", "foo value"),
            ("bar label", "bar value"),
            ("baz label", "baz value"),
            ("qux label", "qux value"),
        ]
    );

    let UserConfiguration::NumberSlider(slider) = &defaults["number_slider_variable"] else {
        panic!("number slider config expected");
    };
    assert_eq!(slider.config.default_value, 50);
    assert_eq!(slider.config.value, 50);
    assert_eq!(slider.config.min, 0);
    assert_eq!(slider.config.max, 100);
    assert_eq!(slider.config.marker_count, Some(10));
    assert!(slider.config.only_stop_on_markers);
    assert!(slider.config.show_markers);

    Ok(())
}

#[test]
fn user_configuration_type_wire_values_and_accessors_cover_all_variants()
-> Result<(), Box<dyn std::error::Error>> {
    let defaults = Workflow::new().get_defaults(INFO_PLIST)?;

    let expected = [
        (
            "textfield_variable",
            UserConfigurationType::TextField,
            "textfield",
            Some("textfield description"),
            Some("textfield label"),
        ),
        (
            "textarea_variable",
            UserConfigurationType::TextArea,
            "textarea",
            Some("textarea description"),
            Some("textarea label"),
        ),
        (
            "checkbox_variable",
            UserConfigurationType::CheckBox,
            "checkbox",
            Some("checkbox description"),
            Some("checkbox label"),
        ),
        (
            "popupbutton_variable",
            UserConfigurationType::Select,
            "popupbutton",
            Some("popupbutton description"),
            Some("popupbutton label"),
        ),
        (
            "filepicker_variable",
            UserConfigurationType::FilePicker,
            "filepicker",
            Some("filepicker description"),
            Some("filepicker label"),
        ),
        (
            "number_slider_variable",
            UserConfigurationType::Slider,
            "slider",
            Some("number slider description"),
            Some("number slider label"),
        ),
    ];

    for (variable, config_type, wire, description, label) in expected {
        let config = &defaults[variable];
        assert_eq!(config_type.as_str(), wire);
        assert_eq!(config.configuration_type(), config_type);
        assert_eq!(config.variable(), variable);
        assert_eq!(config.description(), description);
        assert_eq!(config.label(), label);
    }

    Ok(())
}

#[test]
fn get_user_preferences_returns_raw_plist_values() -> Result<(), Box<dyn std::error::Error>> {
    let preferences: UserPreferences = Workflow::new().get_user_preferences(PREFS_PLIST)?;

    assert_eq!(
        preferences
            .get("textfield_variable")
            .and_then(Value::as_string),
        Some("lorem ipsum dolor")
    );
    assert_eq!(
        preferences
            .get("checkbox_variable")
            .and_then(Value::as_boolean),
        Some(true)
    );
    assert_eq!(
        preferences
            .get("number_slider_variable")
            .and_then(Value::as_signed_integer),
        Some(69)
    );
    assert!(preferences.contains_key("New item"));

    Ok(())
}

#[test]
fn get_user_defaults_merges_preferences_without_losing_defaults()
-> Result<(), Box<dyn std::error::Error>> {
    let user_defaults = Workflow::new().get_user_defaults(INFO_PLIST, PREFS_PLIST)?;

    let UserConfiguration::TextArea(text_area) = &user_defaults["textarea_variable"] else {
        panic!("textarea config expected");
    };
    assert_eq!(text_area.config.default_value, "textarea default");
    assert_eq!(text_area.config.value, "lorem ipsum dolor sit amet");
    assert!(text_area.config.required);
    assert!(!text_area.config.trim);
    assert_eq!(text_area.config.vertical_size, 3);

    let UserConfiguration::CheckBox(check_box) = &user_defaults["checkbox_variable"] else {
        panic!("checkbox config expected");
    };
    assert!(!check_box.config.default_value);
    assert!(check_box.config.value);
    assert_eq!(check_box.config.text.as_deref(), Some("checkbox text"));

    let UserConfiguration::FilePicker(file_picker) = &user_defaults["filepicker_variable"] else {
        panic!("file picker config expected");
    };
    assert_eq!(file_picker.config.default_value, "");
    assert_eq!(file_picker.config.value, "/home/user/Desktop/document.pdf");
    assert_eq!(file_picker.config.filter_mode, 0);

    let UserConfiguration::NumberSlider(slider) = &user_defaults["number_slider_variable"] else {
        panic!("number slider config expected");
    };
    assert_eq!(slider.config.default_value, 50);
    assert_eq!(slider.config.value, 69);

    Ok(())
}

#[test]
fn user_configuration_parsers_return_empty_maps_for_non_matching_schema()
-> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let not_dictionary = dir.path().join("not-dictionary.plist");
    let no_configs = dir.path().join("no-configs.plist");
    let unknown_type = dir.path().join("unknown-type.plist");
    let prefs_not_dictionary = dir.path().join("prefs-not-dictionary.plist");

    write_plist(&not_dictionary, "<string>not a dictionary</string>")?;
    write_plist(&no_configs, "<dict/>")?;
    write_plist(
        &unknown_type,
        r#"<dict>
  <key>userconfigurationconfig</key>
  <array>
    <dict>
      <key>type</key>
      <string>unknown</string>
      <key>variable</key>
      <string>ignored</string>
      <key>config</key>
      <dict/>
    </dict>
  </array>
</dict>"#,
    )?;
    write_plist(&prefs_not_dictionary, "<string>not preferences</string>")?;

    assert!(get_defaults(&not_dictionary)?.is_empty());
    assert!(get_defaults(&no_configs)?.is_empty());
    assert!(get_defaults(&unknown_type)?.is_empty());
    assert!(get_user_preferences(&prefs_not_dictionary)?.is_empty());

    Ok(())
}

#[test]
fn user_configuration_parser_rejects_invalid_config_schema()
-> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let invalid = dir.path().join("invalid.plist");

    write_plist(
        &invalid,
        r#"<dict>
  <key>userconfigurationconfig</key>
  <array>
    <string>not a user configuration dictionary</string>
  </array>
</dict>"#,
    )?;

    assert!(get_defaults(&invalid).is_err());

    Ok(())
}

#[test]
fn user_configuration_parser_accepts_unsigned_integer_slider_values()
-> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let path = dir.path().join("unsigned-slider.plist");
    let mut slider_config = Dictionary::new();
    slider_config.insert("defaultvalue".into(), Value::Integer(50_u64.into()));
    slider_config.insert("minvalue".into(), Value::Integer(0_u64.into()));
    slider_config.insert("maxvalue".into(), Value::Integer(100_u64.into()));
    slider_config.insert("showmarkers".into(), Value::Boolean(true));
    slider_config.insert("onlystoponmarkers".into(), Value::Boolean(true));
    slider_config.insert("markercount".into(), Value::Integer(u64::MAX.into()));

    let mut slider = Dictionary::new();
    slider.insert("type".into(), Value::String(String::from("slider")));
    slider.insert(
        "variable".into(),
        Value::String(String::from("slider_variable")),
    );
    slider.insert("config".into(), Value::Dictionary(slider_config));

    let mut root = Dictionary::new();
    root.insert(
        "userconfigurationconfig".into(),
        Value::Array(vec![Value::Dictionary(slider)]),
    );
    Value::Dictionary(root).to_file_xml(&path)?;

    let defaults = get_defaults(&path)?;
    let UserConfiguration::NumberSlider(slider) = &defaults["slider_variable"] else {
        panic!("slider config expected");
    };
    assert_eq!(slider.config.default_value, 50);
    assert_eq!(slider.config.min, 0);
    assert_eq!(slider.config.max, 100);
    assert_eq!(slider.config.marker_count, None);

    Ok(())
}
