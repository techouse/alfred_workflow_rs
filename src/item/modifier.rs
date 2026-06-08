use std::collections::BTreeMap;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{Error, Result};

use super::Icon;

pub(crate) type ModifierMap = BTreeMap<String, Modifier>;

/// Alfred modifier key.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModifierKey {
    /// Command key.
    Cmd,
    /// Control key.
    Ctrl,
    /// Option/Alt key.
    Alt,
    /// Shift key.
    Shift,
    /// Function key.
    Fn,
}

impl ModifierKey {
    /// Returns the Alfred wire string for this key.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cmd => "cmd",
            Self::Ctrl => "ctrl",
            Self::Alt => "alt",
            Self::Shift => "shift",
            Self::Fn => "fn",
        }
    }
}

impl FromStr for ModifierKey {
    type Err = Error;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value {
            "cmd" => Ok(Self::Cmd),
            "ctrl" => Ok(Self::Ctrl),
            "alt" => Ok(Self::Alt),
            "shift" => Ok(Self::Shift),
            "fn" => Ok(Self::Fn),
            _ => Err(Error::UnknownModifierKey(value.to_owned())),
        }
    }
}

impl Serialize for ModifierKey {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ModifierKey {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_str(&value).map_err(serde::de::Error::custom)
    }
}

/// Alternate item properties shown when modifier keys are held.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Modifier {
    #[serde(skip_serializing_if = "Option::is_none")]
    arg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    subtitle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<Icon>,
    #[serde(default = "default_modifier_valid")]
    valid: bool,
}

impl Modifier {
    /// Creates a modifier with Alfred's default `valid = true`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the modifier argument.
    pub fn with_arg(mut self, arg: impl Into<String>) -> Self {
        self.arg = Some(arg.into());
        self
    }

    /// Sets the modifier subtitle.
    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Sets the modifier icon.
    pub fn with_icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Sets whether the modifier result is valid.
    pub fn with_valid(mut self, valid: bool) -> Self {
        self.valid = valid;
        self
    }

    /// Returns the modifier argument.
    pub fn arg(&self) -> Option<&str> {
        self.arg.as_deref()
    }

    /// Returns the modifier subtitle.
    pub fn subtitle(&self) -> Option<&str> {
        self.subtitle.as_deref()
    }

    /// Returns the modifier icon.
    pub fn icon(&self) -> Option<&Icon> {
        self.icon.as_ref()
    }

    /// Returns whether the modifier result is valid.
    pub fn valid(&self) -> bool {
        self.valid
    }
}

impl Default for Modifier {
    fn default() -> Self {
        Self {
            arg: None,
            subtitle: None,
            icon: None,
            valid: true,
        }
    }
}

fn default_modifier_valid() -> bool {
    true
}

pub(crate) fn canonicalize_modifier_map(mods: ModifierMap) -> Result<ModifierMap> {
    mods.into_iter()
        .map(|(key, modifier)| {
            parse_modifier_keys(&key)
                .and_then(canonical_modifier_key)
                .map(|canonical_key| (canonical_key, modifier))
        })
        .collect()
}

fn parse_modifier_keys(key: &str) -> Result<Vec<ModifierKey>> {
    if key.is_empty() {
        return Err(Error::EmptyModifierKeySet);
    }

    key.split('+').map(ModifierKey::from_str).collect()
}

pub(crate) fn canonical_modifier_key<I>(keys: I) -> Result<String>
where
    I: IntoIterator<Item = ModifierKey>,
{
    let mut names = keys
        .into_iter()
        .map(ModifierKey::as_str)
        .collect::<Vec<_>>();
    if names.is_empty() {
        return Err(Error::EmptyModifierKeySet);
    }

    names.sort_unstable();
    names.dedup();
    Ok(names.join("+"))
}
