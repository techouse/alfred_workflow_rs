use std::collections::BTreeMap;
use std::path::Path;

use plist::{Dictionary, Value};

use crate::{Error, Result, Workflow};

/// Raw user preferences read from `prefs.plist`.
pub type UserPreferences = BTreeMap<String, Value>;

/// Alfred user configuration type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UserConfigurationType {
    /// Text field configuration.
    TextField,
    /// Text area configuration.
    TextArea,
    /// Check box configuration.
    CheckBox,
    /// Popup button/select configuration.
    Select,
    /// File picker configuration.
    FilePicker,
    /// Number slider configuration.
    Slider,
}

impl UserConfigurationType {
    /// Returns Alfred's plist wire value.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TextField => "textfield",
            Self::TextArea => "textarea",
            Self::CheckBox => "checkbox",
            Self::Select => "popupbutton",
            Self::FilePicker => "filepicker",
            Self::Slider => "slider",
        }
    }

    fn from_wire(value: &str) -> Option<Self> {
        match value {
            "textfield" => Some(Self::TextField),
            "textarea" => Some(Self::TextArea),
            "checkbox" => Some(Self::CheckBox),
            "popupbutton" => Some(Self::Select),
            "filepicker" => Some(Self::FilePicker),
            "slider" => Some(Self::Slider),
            _ => None,
        }
    }
}

/// Text field user configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextFieldUserConfiguration {
    /// Variable name.
    pub variable: String,
    /// Optional description.
    pub description: Option<String>,
    /// Optional label.
    pub label: Option<String>,
    /// Variant-specific config.
    pub config: TextFieldConfiguration,
}

/// Text field config.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextFieldConfiguration {
    /// Default value from `info.plist`.
    pub default_value: String,
    /// Current value after optional `prefs.plist` merge.
    pub value: String,
    /// Optional placeholder text.
    pub placeholder: Option<String>,
    /// Whether the field is required.
    pub required: bool,
    /// Whether Alfred should trim whitespace.
    pub trim: bool,
}

/// Text area user configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextAreaUserConfiguration {
    /// Variable name.
    pub variable: String,
    /// Optional description.
    pub description: Option<String>,
    /// Optional label.
    pub label: Option<String>,
    /// Variant-specific config.
    pub config: TextAreaConfiguration,
}

/// Text area config.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextAreaConfiguration {
    /// Default value from `info.plist`.
    pub default_value: String,
    /// Current value after optional `prefs.plist` merge.
    pub value: String,
    /// Whether the field is required.
    pub required: bool,
    /// Whether Alfred should trim whitespace.
    pub trim: bool,
    /// Text area vertical size.
    pub vertical_size: i64,
}

/// Check box user configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckBoxUserConfiguration {
    /// Variable name.
    pub variable: String,
    /// Optional description.
    pub description: Option<String>,
    /// Optional label.
    pub label: Option<String>,
    /// Variant-specific config.
    pub config: CheckBoxConfiguration,
}

/// Check box config.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckBoxConfiguration {
    /// Default value from `info.plist`.
    pub default_value: bool,
    /// Current value after optional `prefs.plist` merge.
    pub value: bool,
    /// Whether the field is required.
    pub required: bool,
    /// Optional text shown next to the check box.
    pub text: Option<String>,
}

/// Popup button/select pair.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectPair {
    /// Display label.
    pub label: String,
    /// Stored value.
    pub value: String,
}

/// Popup button/select user configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectUserConfiguration {
    /// Variable name.
    pub variable: String,
    /// Optional description.
    pub description: Option<String>,
    /// Optional label.
    pub label: Option<String>,
    /// Variant-specific config.
    pub config: SelectConfiguration,
}

/// Popup button/select config.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectConfiguration {
    /// Default value from `info.plist`.
    pub default_value: String,
    /// Current value after optional `prefs.plist` merge.
    pub value: String,
    /// Ordered label/value pairs.
    pub pairs: Vec<SelectPair>,
}

/// File picker user configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FilePickerUserConfiguration {
    /// Variable name.
    pub variable: String,
    /// Optional description.
    pub description: Option<String>,
    /// Optional label.
    pub label: Option<String>,
    /// Variant-specific config.
    pub config: FilePickerConfiguration,
}

/// File picker config.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FilePickerConfiguration {
    /// Default value from `info.plist`.
    pub default_value: String,
    /// Current value after optional `prefs.plist` merge.
    pub value: String,
    /// Whether the field is required.
    pub required: bool,
    /// Optional placeholder text.
    pub placeholder: Option<String>,
    /// Alfred filter mode.
    pub filter_mode: i64,
}

/// Number slider user configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NumberSliderUserConfiguration {
    /// Variable name.
    pub variable: String,
    /// Optional description.
    pub description: Option<String>,
    /// Optional label.
    pub label: Option<String>,
    /// Variant-specific config.
    pub config: NumberSliderConfiguration,
}

/// Number slider config.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NumberSliderConfiguration {
    /// Default value from `info.plist`.
    pub default_value: i64,
    /// Current value after optional `prefs.plist` merge.
    pub value: i64,
    /// Minimum slider value.
    pub min: i64,
    /// Maximum slider value.
    pub max: i64,
    /// Whether Alfred should show markers.
    pub show_markers: bool,
    /// Whether Alfred should stop only on markers.
    pub only_stop_on_markers: bool,
    /// Optional marker count.
    pub marker_count: Option<i64>,
}

/// Alfred user configuration item.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UserConfiguration {
    /// Text field configuration.
    TextField(TextFieldUserConfiguration),
    /// Text area configuration.
    TextArea(TextAreaUserConfiguration),
    /// Check box configuration.
    CheckBox(CheckBoxUserConfiguration),
    /// Popup button/select configuration.
    Select(SelectUserConfiguration),
    /// File picker configuration.
    FilePicker(FilePickerUserConfiguration),
    /// Number slider configuration.
    NumberSlider(NumberSliderUserConfiguration),
}

impl UserConfiguration {
    /// Returns the Alfred configuration type.
    pub fn configuration_type(&self) -> UserConfigurationType {
        match self {
            Self::TextField(_) => UserConfigurationType::TextField,
            Self::TextArea(_) => UserConfigurationType::TextArea,
            Self::CheckBox(_) => UserConfigurationType::CheckBox,
            Self::Select(_) => UserConfigurationType::Select,
            Self::FilePicker(_) => UserConfigurationType::FilePicker,
            Self::NumberSlider(_) => UserConfigurationType::Slider,
        }
    }

    /// Returns the variable name.
    pub fn variable(&self) -> &str {
        match self {
            Self::TextField(config) => &config.variable,
            Self::TextArea(config) => &config.variable,
            Self::CheckBox(config) => &config.variable,
            Self::Select(config) => &config.variable,
            Self::FilePicker(config) => &config.variable,
            Self::NumberSlider(config) => &config.variable,
        }
    }

    /// Returns the optional description.
    pub fn description(&self) -> Option<&str> {
        match self {
            Self::TextField(config) => config.description.as_deref(),
            Self::TextArea(config) => config.description.as_deref(),
            Self::CheckBox(config) => config.description.as_deref(),
            Self::Select(config) => config.description.as_deref(),
            Self::FilePicker(config) => config.description.as_deref(),
            Self::NumberSlider(config) => config.description.as_deref(),
        }
    }

    /// Returns the optional label.
    pub fn label(&self) -> Option<&str> {
        match self {
            Self::TextField(config) => config.label.as_deref(),
            Self::TextArea(config) => config.label.as_deref(),
            Self::CheckBox(config) => config.label.as_deref(),
            Self::Select(config) => config.label.as_deref(),
            Self::FilePicker(config) => config.label.as_deref(),
            Self::NumberSlider(config) => config.label.as_deref(),
        }
    }

    fn with_preference_value(self, value: Option<&Value>) -> Self {
        match self {
            Self::TextField(mut item) => {
                item.config.value = value
                    .and_then(Value::as_string)
                    .unwrap_or(&item.config.default_value)
                    .to_owned();
                Self::TextField(item)
            }
            Self::TextArea(mut item) => {
                item.config.value = value
                    .and_then(Value::as_string)
                    .unwrap_or(&item.config.default_value)
                    .to_owned();
                Self::TextArea(item)
            }
            Self::CheckBox(mut item) => {
                item.config.value = value
                    .and_then(Value::as_boolean)
                    .unwrap_or(item.config.default_value);
                Self::CheckBox(item)
            }
            Self::Select(mut item) => {
                item.config.value = value
                    .and_then(Value::as_string)
                    .unwrap_or(&item.config.default_value)
                    .to_owned();
                Self::Select(item)
            }
            Self::FilePicker(mut item) => {
                item.config.value = value
                    .and_then(Value::as_string)
                    .unwrap_or(&item.config.default_value)
                    .to_owned();
                Self::FilePicker(item)
            }
            Self::NumberSlider(mut item) => {
                item.config.value = value
                    .and_then(plist_integer)
                    .unwrap_or(item.config.default_value);
                Self::NumberSlider(item)
            }
        }
    }
}

impl Workflow {
    /// Reads default user configuration values from `info.plist`.
    pub fn get_defaults(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<BTreeMap<String, UserConfiguration>> {
        get_defaults(path)
    }

    /// Reads raw user preferences from `prefs.plist`.
    pub fn get_user_preferences(&self, path: impl AsRef<Path>) -> Result<UserPreferences> {
        get_user_preferences(path)
    }

    /// Reads and merges `info.plist` defaults with `prefs.plist` preferences.
    pub fn get_user_defaults(
        &self,
        info_path: impl AsRef<Path>,
        prefs_path: impl AsRef<Path>,
    ) -> Result<BTreeMap<String, UserConfiguration>> {
        get_user_defaults(info_path, prefs_path)
    }
}

/// Reads default user configuration values from `info.plist`.
pub fn get_defaults(path: impl AsRef<Path>) -> Result<BTreeMap<String, UserConfiguration>> {
    let Some(value) = read_plist(path)? else {
        return Ok(BTreeMap::new());
    };
    let Some(info) = value.as_dictionary() else {
        return Ok(BTreeMap::new());
    };
    let Some(configs) = info
        .get("userconfigurationconfig")
        .and_then(Value::as_array)
    else {
        return Ok(BTreeMap::new());
    };

    let mut defaults = BTreeMap::new();
    for config in configs {
        if let Some(config) = parse_user_configuration(config)? {
            defaults.insert(config.variable().to_owned(), config);
        }
    }

    Ok(defaults)
}

/// Reads raw user preferences from `prefs.plist`.
pub fn get_user_preferences(path: impl AsRef<Path>) -> Result<UserPreferences> {
    let Some(value) = read_plist(path)? else {
        return Ok(BTreeMap::new());
    };
    let Some(prefs) = value.as_dictionary() else {
        return Ok(BTreeMap::new());
    };

    Ok(prefs
        .iter()
        .map(|(key, value)| (key.to_owned(), value.clone()))
        .collect())
}

/// Reads and merges `info.plist` defaults with `prefs.plist` preferences.
pub fn get_user_defaults(
    info_path: impl AsRef<Path>,
    prefs_path: impl AsRef<Path>,
) -> Result<BTreeMap<String, UserConfiguration>> {
    let preferences = get_user_preferences(prefs_path)?;
    let defaults = get_defaults(info_path)?;

    Ok(defaults
        .into_iter()
        .map(|(key, config)| {
            let config = config.with_preference_value(preferences.get(&key));
            (key, config)
        })
        .collect())
}

fn read_plist(path: impl AsRef<Path>) -> Result<Option<Value>> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(None);
    }

    Value::from_file(path)
        .map(Some)
        .map_err(|error| Error::UserConfiguration(error.to_string()))
}

fn parse_user_configuration(value: &Value) -> Result<Option<UserConfiguration>> {
    let dict = required_dictionary(value, "user configuration")?;
    let Some(configuration_type) = dict
        .get("type")
        .and_then(Value::as_string)
        .and_then(UserConfigurationType::from_wire)
    else {
        return Ok(None);
    };

    let variable = required_string(dict, "variable")?;
    let description = optional_string(dict, "description");
    let label = optional_string(dict, "label");
    let config = required_dictionary_key(dict, "config")?;

    Ok(Some(match configuration_type {
        UserConfigurationType::TextField => {
            UserConfiguration::TextField(TextFieldUserConfiguration {
                variable,
                description,
                label,
                config: TextFieldConfiguration {
                    default_value: required_default_string(config)?,
                    value: required_default_string(config)?,
                    placeholder: optional_string(config, "placeholder"),
                    required: required_bool(config, "required")?,
                    trim: required_bool(config, "trim")?,
                },
            })
        }
        UserConfigurationType::TextArea => UserConfiguration::TextArea(TextAreaUserConfiguration {
            variable,
            description,
            label,
            config: TextAreaConfiguration {
                default_value: required_default_string(config)?,
                value: required_default_string(config)?,
                required: required_bool(config, "required")?,
                trim: required_bool(config, "trim")?,
                vertical_size: required_integer(config, "verticalsize")?,
            },
        }),
        UserConfigurationType::CheckBox => UserConfiguration::CheckBox(CheckBoxUserConfiguration {
            variable,
            description,
            label,
            config: CheckBoxConfiguration {
                default_value: required_default_bool(config)?,
                value: required_default_bool(config)?,
                required: required_bool(config, "required")?,
                text: optional_string(config, "text"),
            },
        }),
        UserConfigurationType::Select => UserConfiguration::Select(SelectUserConfiguration {
            variable,
            description,
            label,
            config: SelectConfiguration {
                default_value: required_default_string(config)?,
                value: required_default_string(config)?,
                pairs: required_select_pairs(config)?,
            },
        }),
        UserConfigurationType::FilePicker => {
            UserConfiguration::FilePicker(FilePickerUserConfiguration {
                variable,
                description,
                label,
                config: FilePickerConfiguration {
                    default_value: required_default_string(config)?,
                    value: required_default_string(config)?,
                    required: required_bool(config, "required")?,
                    placeholder: optional_string(config, "placeholder"),
                    filter_mode: required_integer(config, "filtermode")?,
                },
            })
        }
        UserConfigurationType::Slider => {
            UserConfiguration::NumberSlider(NumberSliderUserConfiguration {
                variable,
                description,
                label,
                config: NumberSliderConfiguration {
                    default_value: required_default_integer(config)?,
                    value: required_default_integer(config)?,
                    min: required_integer(config, "minvalue")?,
                    max: required_integer(config, "maxvalue")?,
                    show_markers: required_bool(config, "showmarkers")?,
                    only_stop_on_markers: required_bool(config, "onlystoponmarkers")?,
                    marker_count: optional_integer(config, "markercount"),
                },
            })
        }
    }))
}

fn required_dictionary<'a>(value: &'a Value, context: &str) -> Result<&'a Dictionary> {
    value
        .as_dictionary()
        .ok_or_else(|| schema_error(format!("{context} must be a dictionary")))
}

fn required_dictionary_key<'a>(dict: &'a Dictionary, key: &str) -> Result<&'a Dictionary> {
    dict.get(key)
        .and_then(Value::as_dictionary)
        .ok_or_else(|| schema_error(format!("`{key}` must be a dictionary")))
}

fn required_string(dict: &Dictionary, key: &str) -> Result<String> {
    dict.get(key)
        .and_then(Value::as_string)
        .map(str::to_owned)
        .ok_or_else(|| schema_error(format!("`{key}` must be a string")))
}

fn optional_string(dict: &Dictionary, key: &str) -> Option<String> {
    dict.get(key).and_then(Value::as_string).map(str::to_owned)
}

fn required_bool(dict: &Dictionary, key: &str) -> Result<bool> {
    dict.get(key)
        .and_then(Value::as_boolean)
        .ok_or_else(|| schema_error(format!("`{key}` must be a boolean")))
}

fn required_integer(dict: &Dictionary, key: &str) -> Result<i64> {
    dict.get(key)
        .and_then(plist_integer)
        .ok_or_else(|| schema_error(format!("`{key}` must be an integer")))
}

fn optional_integer(dict: &Dictionary, key: &str) -> Option<i64> {
    dict.get(key).and_then(plist_integer)
}

fn required_default_string(dict: &Dictionary) -> Result<String> {
    default_value(dict)
        .and_then(Value::as_string)
        .map(str::to_owned)
        .ok_or_else(|| schema_error("default value must be a string"))
}

fn required_default_bool(dict: &Dictionary) -> Result<bool> {
    default_value(dict)
        .and_then(Value::as_boolean)
        .ok_or_else(|| schema_error("default value must be a boolean"))
}

fn required_default_integer(dict: &Dictionary) -> Result<i64> {
    default_value(dict)
        .and_then(plist_integer)
        .ok_or_else(|| schema_error("default value must be an integer"))
}

fn default_value(dict: &Dictionary) -> Option<&Value> {
    dict.get("defaultvalue").or_else(|| dict.get("default"))
}

fn required_select_pairs(dict: &Dictionary) -> Result<Vec<SelectPair>> {
    let pairs = dict
        .get("pairs")
        .and_then(Value::as_array)
        .ok_or_else(|| schema_error("`pairs` must be an array"))?;

    pairs
        .iter()
        .map(|pair| {
            let pair = pair
                .as_array()
                .ok_or_else(|| schema_error("select pair must be an array"))?;
            let label = pair
                .first()
                .and_then(Value::as_string)
                .ok_or_else(|| schema_error("select pair label must be a string"))?;
            let value = pair
                .last()
                .and_then(Value::as_string)
                .ok_or_else(|| schema_error("select pair value must be a string"))?;

            Ok(SelectPair {
                label: label.to_owned(),
                value: value.to_owned(),
            })
        })
        .collect()
}

fn plist_integer(value: &Value) -> Option<i64> {
    value.as_signed_integer().or_else(|| {
        value
            .as_unsigned_integer()
            .and_then(|value| i64::try_from(value).ok())
    })
}

fn schema_error(message: impl Into<String>) -> Error {
    Error::UserConfiguration(message.into())
}
