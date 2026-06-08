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

    pub(crate) fn from_wire(value: &str) -> Option<Self> {
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
