use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};

/// Alfred copy and large-type text metadata.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct ItemText {
    copy: String,
    #[serde(rename = "largetype", default)]
    large_type: Option<String>,
}

impl Serialize for ItemText {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(1 + usize::from(self.large_type.is_some())))?;
        map.serialize_entry("copy", &self.copy)?;
        if let Some(large_type) = &self.large_type {
            map.serialize_entry("largetype", large_type)?;
        }
        map.end()
    }
}

impl ItemText {
    /// Creates text metadata with a copy value.
    pub fn new(copy: impl Into<String>) -> Self {
        Self {
            copy: copy.into(),
            large_type: None,
        }
    }

    /// Sets the optional Alfred `largetype` value.
    pub fn with_large_type(mut self, large_type: impl Into<String>) -> Self {
        self.large_type = Some(large_type.into());
        self
    }

    /// Returns the copy text.
    pub fn copy(&self) -> &str {
        &self.copy
    }

    /// Returns the optional large-type text.
    pub fn large_type(&self) -> Option<&str> {
        self.large_type.as_deref()
    }
}
