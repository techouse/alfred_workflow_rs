use serde::{Deserialize, Serialize};

/// Alfred copy and large-type text metadata.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ItemText {
    copy: String,
    #[serde(rename = "largetype", skip_serializing_if = "Option::is_none")]
    large_type: Option<String>,
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
