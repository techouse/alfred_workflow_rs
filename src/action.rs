use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{Error, Result};

/// Alfred action value.
///
/// Alfred accepts action payloads as a string, a list, or a typed object.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Action {
    /// A single string action value.
    String(String),
    /// A list of nested action values.
    List(Vec<Action>),
    /// A typed Alfred action object.
    Typed(TypedAction),
}

impl Action {
    fn from_json_value(value: Value) -> std::result::Result<Self, String> {
        match value {
            Value::String(value) => Ok(Self::String(value)),
            Value::Array(values) => values
                .into_iter()
                .map(Self::from_json_value)
                .collect::<std::result::Result<Vec<_>, _>>()
                .map(Self::List),
            Value::Object(_) => serde_json::from_value::<TypedAction>(value)
                .map(Self::Typed)
                .map_err(|error| error.to_string()),
            Value::Null | Value::Bool(_) | Value::Number(_) => {
                Err("invalid action; expected string, list, or typed object".to_owned())
            }
        }
    }
}

impl From<String> for Action {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for Action {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<Vec<Action>> for Action {
    fn from(value: Vec<Action>) -> Self {
        Self::List(value)
    }
}

impl From<Vec<String>> for Action {
    fn from(value: Vec<String>) -> Self {
        Self::List(value.into_iter().map(Self::String).collect())
    }
}

impl<'a> From<Vec<&'a str>> for Action {
    fn from(value: Vec<&'a str>) -> Self {
        Self::List(value.into_iter().map(Self::from).collect())
    }
}

impl From<TypedAction> for Action {
    fn from(value: TypedAction) -> Self {
        Self::Typed(value)
    }
}

impl Serialize for Action {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::String(value) => value.serialize(serializer),
            Self::List(values) => values.serialize(serializer),
            Self::Typed(value) => value.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Action {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        Self::from_json_value(value).map_err(serde::de::Error::custom)
    }
}

/// Typed Alfred action object.
///
/// At least one field must be present.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TypedAction {
    text: Option<ActionText>,
    url: Option<String>,
    file: Option<String>,
    auto: Option<String>,
}

impl TypedAction {
    /// Creates a typed action from optional fields.
    pub fn try_new(
        text: Option<ActionText>,
        url: Option<String>,
        file: Option<String>,
        auto: Option<String>,
    ) -> Result<Self> {
        if text.is_none() && url.is_none() && file.is_none() && auto.is_none() {
            return Err(Error::EmptyTypedAction);
        }

        Ok(Self {
            text,
            url,
            file,
            auto,
        })
    }

    /// Creates a typed action with only a `text` field.
    pub fn text(text: impl Into<ActionText>) -> Self {
        Self {
            text: Some(text.into()),
            url: None,
            file: None,
            auto: None,
        }
    }

    /// Creates a typed action with only a `url` field.
    pub fn url(url: impl Into<String>) -> Self {
        Self {
            text: None,
            url: Some(url.into()),
            file: None,
            auto: None,
        }
    }

    /// Creates a typed action with only a `file` field.
    pub fn file(file: impl Into<String>) -> Self {
        Self {
            text: None,
            url: None,
            file: Some(file.into()),
            auto: None,
        }
    }

    /// Creates a typed action with only an `auto` field.
    pub fn auto(auto: impl Into<String>) -> Self {
        Self {
            text: None,
            url: None,
            file: None,
            auto: Some(auto.into()),
        }
    }

    /// Sets or replaces the `text` field.
    pub fn with_text(mut self, text: impl Into<ActionText>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Sets or replaces the `url` field.
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Sets or replaces the `file` field.
    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.file = Some(file.into());
        self
    }

    /// Sets or replaces the `auto` field.
    pub fn with_auto(mut self, auto: impl Into<String>) -> Self {
        self.auto = Some(auto.into());
        self
    }

    /// Returns the `text` field.
    pub fn text_value(&self) -> Option<&ActionText> {
        self.text.as_ref()
    }

    /// Returns the `url` field.
    pub fn url_value(&self) -> Option<&str> {
        self.url.as_deref()
    }

    /// Returns the `file` field.
    pub fn file_value(&self) -> Option<&str> {
        self.file.as_deref()
    }

    /// Returns the `auto` field.
    pub fn auto_value(&self) -> Option<&str> {
        self.auto.as_deref()
    }
}

impl<'de> Deserialize<'de> for TypedAction {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire {
            text: Option<ActionText>,
            url: Option<String>,
            file: Option<String>,
            auto: Option<String>,
        }

        let wire = Wire::deserialize(deserializer)?;
        Self::try_new(wire.text, wire.url, wire.file, wire.auto).map_err(serde::de::Error::custom)
    }
}

/// Text payload used by a typed Alfred action.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActionText {
    /// A single text value.
    String(String),
    /// A list of text values.
    List(Vec<String>),
}

impl From<String> for ActionText {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for ActionText {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<Vec<String>> for ActionText {
    fn from(value: Vec<String>) -> Self {
        Self::List(value)
    }
}

impl<'a> From<Vec<&'a str>> for ActionText {
    fn from(value: Vec<&'a str>) -> Self {
        Self::List(value.into_iter().map(str::to_owned).collect())
    }
}

impl Serialize for ActionText {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::String(value) => value.serialize(serializer),
            Self::List(values) => values.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for ActionText {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        match value {
            Value::String(value) => Ok(Self::String(value)),
            Value::Array(values) => values
                .into_iter()
                .map(|value| match value {
                    Value::String(value) => Ok(value),
                    _ => Err("typed action text list must contain only strings".to_owned()),
                })
                .collect::<std::result::Result<Vec<_>, _>>()
                .map(Self::List)
                .map_err(serde::de::Error::custom),
            _ => Err(serde::de::Error::custom(
                "typed action text must be a string or list",
            )),
        }
    }
}
