use alfred_workflow_rs::{UserConfiguration, UserConfigurationType, UserPreferences, Workflow};
use plist::Value;
use pretty_assertions::assert_eq;

const INFO_PLIST: &str = "tests/fixtures/info.plist";
const PREFS_PLIST: &str = "tests/fixtures/prefs.plist";

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
