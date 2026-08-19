use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};

/// Alfred icon type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IconType {
    /// Treat the path as a file whose icon should be displayed.
    FileIcon,
    /// Treat the path as a UTI or file extension.
    FileType,
}

impl IconType {
    /// Returns the Alfred wire string for this icon type.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FileIcon => "fileicon",
            Self::FileType => "filetype",
        }
    }
}

impl Serialize for IconType {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for IconType {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "fileicon" => Ok(Self::FileIcon),
            "filetype" => Ok(Self::FileType),
            _ => Err(serde::de::Error::custom(format!(
                "unknown icon type `{value}`"
            ))),
        }
    }
}

/// Alfred result icon metadata.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct Icon {
    path: String,
    #[serde(rename = "type", default)]
    icon_type: Option<IconType>,
}

impl Serialize for Icon {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(1 + usize::from(self.icon_type.is_some())))?;
        map.serialize_entry("path", &self.path)?;
        if let Some(icon_type) = self.icon_type {
            map.serialize_entry("type", &icon_type)?;
        }
        map.end()
    }
}

impl Icon {
    /// Creates an icon from a path.
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            icon_type: None,
        }
    }

    /// Sets the optional Alfred icon type.
    pub fn with_type(mut self, icon_type: IconType) -> Self {
        self.icon_type = Some(icon_type);
        self
    }

    /// Returns the icon path.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns the optional Alfred icon type.
    pub fn icon_type(&self) -> Option<IconType> {
        self.icon_type
    }
}
