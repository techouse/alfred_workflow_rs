use std::collections::BTreeMap;
use std::str::FromStr;

use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};

use crate::{Action, Error, Result};

type ModifierMap = BTreeMap<String, Modifier>;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ItemType {
    #[default]
    Default,
    File,
    FileSkipcheck,
}

impl ItemType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::File => "file",
            Self::FileSkipcheck => "file:skipcheck",
        }
    }
}

impl Serialize for ItemType {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ItemType {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "default" => Ok(Self::Default),
            "file" => Ok(Self::File),
            "file:skipcheck" => Ok(Self::FileSkipcheck),
            _ => Err(serde::de::Error::custom(format!(
                "unknown item type `{value}`"
            ))),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IconType {
    FileIcon,
    FileType,
}

impl IconType {
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Icon {
    path: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    icon_type: Option<IconType>,
}

impl Icon {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            icon_type: None,
        }
    }

    pub fn with_type(mut self, icon_type: IconType) -> Self {
        self.icon_type = Some(icon_type);
        self
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn icon_type(&self) -> Option<IconType> {
        self.icon_type
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ItemText {
    copy: String,
    #[serde(rename = "largetype", skip_serializing_if = "Option::is_none")]
    large_type: Option<String>,
}

impl ItemText {
    pub fn new(copy: impl Into<String>) -> Self {
        Self {
            copy: copy.into(),
            large_type: None,
        }
    }

    pub fn with_large_type(mut self, large_type: impl Into<String>) -> Self {
        self.large_type = Some(large_type.into());
        self
    }

    pub fn copy(&self) -> &str {
        &self.copy
    }

    pub fn large_type(&self) -> Option<&str> {
        self.large_type.as_deref()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModifierKey {
    Cmd,
    Ctrl,
    Alt,
    Shift,
    Fn,
}

impl ModifierKey {
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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_arg(mut self, arg: impl Into<String>) -> Self {
        self.arg = Some(arg.into());
        self
    }

    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn with_icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn with_valid(mut self, valid: bool) -> Self {
        self.valid = valid;
        self
    }

    pub fn arg(&self) -> Option<&str> {
        self.arg.as_deref()
    }

    pub fn subtitle(&self) -> Option<&str> {
        self.subtitle.as_deref()
    }

    pub fn icon(&self) -> Option<&Icon> {
        self.icon.as_ref()
    }

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Item {
    title: String,
    item_type: ItemType,
    valid: bool,
    subtitle: Option<String>,
    arg: Option<String>,
    autocomplete: Option<String>,
    uid: Option<String>,
    icon: Option<Icon>,
    text: Option<ItemText>,
    quick_look_url: Option<String>,
    match_text: Option<String>,
    mods: Option<ModifierMap>,
    action: Option<Action>,
}

impl Item {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            item_type: ItemType::Default,
            valid: false,
            subtitle: None,
            arg: None,
            autocomplete: None,
            uid: None,
            icon: None,
            text: None,
            quick_look_url: None,
            match_text: None,
            mods: None,
            action: None,
        }
    }

    pub fn with_arg(title: impl Into<String>, arg: impl Into<String>) -> Self {
        Self::new(title).set_arg(arg)
    }

    pub fn with_action(title: impl Into<String>, action: impl Into<Action>) -> Self {
        Self::new(title).set_action(action)
    }

    pub fn builder(title: impl Into<String>) -> ItemBuilder {
        ItemBuilder::new(title)
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn item_type(&self) -> ItemType {
        self.item_type
    }

    pub fn valid(&self) -> bool {
        self.valid
    }

    pub fn subtitle(&self) -> Option<&str> {
        self.subtitle.as_deref()
    }

    pub fn arg(&self) -> Option<&str> {
        self.arg.as_deref()
    }

    pub fn autocomplete(&self) -> Option<&str> {
        self.autocomplete.as_deref()
    }

    pub fn uid(&self) -> Option<&str> {
        self.uid.as_deref()
    }

    pub fn icon(&self) -> Option<&Icon> {
        self.icon.as_ref()
    }

    pub fn text(&self) -> Option<&ItemText> {
        self.text.as_ref()
    }

    pub fn quick_look_url(&self) -> Option<&str> {
        self.quick_look_url.as_deref()
    }

    pub fn match_text(&self) -> Option<&str> {
        self.match_text.as_deref()
    }

    pub fn modifiers(&self) -> Option<&BTreeMap<String, Modifier>> {
        self.mods.as_ref()
    }

    pub fn action(&self) -> Option<&Action> {
        self.action.as_ref()
    }

    pub fn set_item_type(mut self, item_type: ItemType) -> Self {
        self.item_type = item_type;
        self
    }

    pub fn set_valid(mut self, valid: bool) -> Self {
        self.valid = valid;
        self
    }

    pub fn set_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn set_arg(mut self, arg: impl Into<String>) -> Self {
        self.arg = Some(arg.into());
        self.action = None;
        self
    }

    pub fn set_autocomplete(mut self, autocomplete: impl Into<String>) -> Self {
        self.autocomplete = Some(autocomplete.into());
        self
    }

    pub fn set_uid(mut self, uid: impl Into<String>) -> Self {
        self.uid = Some(uid.into());
        self
    }

    pub fn set_icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn set_text(mut self, text: ItemText) -> Self {
        self.text = Some(text);
        self
    }

    pub fn set_quick_look_url(mut self, quick_look_url: impl Into<String>) -> Self {
        self.quick_look_url = Some(quick_look_url.into());
        self
    }

    pub fn set_match_text(mut self, match_text: impl Into<String>) -> Self {
        self.match_text = Some(match_text.into());
        self
    }

    pub fn try_set_modifier<I>(mut self, keys: I, modifier: Modifier) -> Result<Self>
    where
        I: IntoIterator<Item = ModifierKey>,
    {
        let key = canonical_modifier_key(keys)?;
        self.mods
            .get_or_insert_with(BTreeMap::new)
            .insert(key, modifier);
        Ok(self)
    }

    pub fn set_action(mut self, action: impl Into<Action>) -> Self {
        self.action = Some(action.into());
        self.arg = None;
        self
    }

    fn try_from_parts(parts: ItemParts) -> Result<Self> {
        if parts.arg.is_some() && parts.action.is_some() {
            return Err(Error::ArgAndAction);
        }

        Ok(Self {
            title: parts.title,
            item_type: parts.item_type,
            valid: parts.valid,
            subtitle: parts.subtitle,
            arg: parts.arg,
            autocomplete: parts.autocomplete,
            uid: parts.uid,
            icon: parts.icon,
            text: parts.text,
            quick_look_url: parts.quick_look_url,
            match_text: parts.match_text,
            mods: parts.mods,
            action: parts.action,
        })
    }

    pub(crate) fn serializer(&self, include_uid: bool) -> ItemSerializer<'_> {
        ItemSerializer {
            item: self,
            include_uid,
        }
    }
}

#[derive(Default)]
struct ItemParts {
    title: String,
    item_type: ItemType,
    valid: bool,
    subtitle: Option<String>,
    arg: Option<String>,
    autocomplete: Option<String>,
    uid: Option<String>,
    icon: Option<Icon>,
    text: Option<ItemText>,
    quick_look_url: Option<String>,
    match_text: Option<String>,
    mods: Option<ModifierMap>,
    action: Option<Action>,
}

pub struct ItemBuilder {
    parts: ItemParts,
}

impl ItemBuilder {
    fn new(title: impl Into<String>) -> Self {
        Self {
            parts: ItemParts {
                title: title.into(),
                ..ItemParts::default()
            },
        }
    }

    pub fn item_type(mut self, item_type: ItemType) -> Self {
        self.parts.item_type = item_type;
        self
    }

    pub fn valid(mut self, valid: bool) -> Self {
        self.parts.valid = valid;
        self
    }

    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.parts.subtitle = Some(subtitle.into());
        self
    }

    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.parts.arg = Some(arg.into());
        self
    }

    pub fn autocomplete(mut self, autocomplete: impl Into<String>) -> Self {
        self.parts.autocomplete = Some(autocomplete.into());
        self
    }

    pub fn uid(mut self, uid: impl Into<String>) -> Self {
        self.parts.uid = Some(uid.into());
        self
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.parts.icon = Some(icon);
        self
    }

    pub fn text(mut self, text: ItemText) -> Self {
        self.parts.text = Some(text);
        self
    }

    pub fn quick_look_url(mut self, quick_look_url: impl Into<String>) -> Self {
        self.parts.quick_look_url = Some(quick_look_url.into());
        self
    }

    pub fn match_text(mut self, match_text: impl Into<String>) -> Self {
        self.parts.match_text = Some(match_text.into());
        self
    }

    pub fn try_modifier<I>(mut self, keys: I, modifier: Modifier) -> Result<Self>
    where
        I: IntoIterator<Item = ModifierKey>,
    {
        let key = canonical_modifier_key(keys)?;
        self.parts
            .mods
            .get_or_insert_with(BTreeMap::new)
            .insert(key, modifier);
        Ok(self)
    }

    pub fn action(mut self, action: impl Into<Action>) -> Self {
        self.parts.action = Some(action.into());
        self
    }

    pub fn build(self) -> Result<Item> {
        Item::try_from_parts(self.parts)
    }
}

impl Serialize for Item {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.serializer(true).serialize(serializer)
    }
}

pub(crate) struct ItemSerializer<'a> {
    item: &'a Item,
    include_uid: bool,
}

impl Serialize for ItemSerializer<'_> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let item = self.item;
        let mut len = 3;
        len += usize::from(item.subtitle.is_some());
        len += usize::from(item.arg.is_some());
        len += usize::from(item.autocomplete.is_some());
        len += usize::from(self.include_uid && item.uid.is_some());
        len += usize::from(item.icon.is_some());
        len += usize::from(item.text.is_some());
        len += usize::from(item.quick_look_url.is_some());
        len += usize::from(item.match_text.is_some());
        len += usize::from(item.mods.is_some());
        len += usize::from(item.action.is_some());

        let mut map = serializer.serialize_map(Some(len))?;
        map.serialize_entry("title", &item.title)?;
        map.serialize_entry("type", &item.item_type)?;
        map.serialize_entry("valid", &item.valid)?;
        if let Some(subtitle) = &item.subtitle {
            map.serialize_entry("subtitle", subtitle)?;
        }
        if let Some(arg) = &item.arg {
            map.serialize_entry("arg", arg)?;
        }
        if let Some(autocomplete) = &item.autocomplete {
            map.serialize_entry("autocomplete", autocomplete)?;
        }
        if self.include_uid
            && let Some(uid) = &item.uid
        {
            map.serialize_entry("uid", uid)?;
        }
        if let Some(icon) = &item.icon {
            map.serialize_entry("icon", icon)?;
        }
        if let Some(text) = &item.text {
            map.serialize_entry("text", text)?;
        }
        if let Some(quick_look_url) = &item.quick_look_url {
            map.serialize_entry("quicklookurl", quick_look_url)?;
        }
        if let Some(match_text) = &item.match_text {
            map.serialize_entry("match", match_text)?;
        }
        if let Some(mods) = &item.mods {
            map.serialize_entry("mods", mods)?;
        }
        if let Some(action) = &item.action {
            map.serialize_entry("action", action)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for Item {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire {
            title: String,
            #[serde(rename = "type", default)]
            item_type: ItemType,
            #[serde(default)]
            valid: bool,
            subtitle: Option<String>,
            arg: Option<String>,
            autocomplete: Option<String>,
            uid: Option<String>,
            icon: Option<Icon>,
            text: Option<ItemText>,
            #[serde(rename = "quicklookurl")]
            quick_look_url: Option<String>,
            #[serde(rename = "match")]
            match_text: Option<String>,
            mods: Option<ModifierMap>,
            action: Option<Action>,
        }

        let wire = Wire::deserialize(deserializer)?;
        Item::try_from_parts(ItemParts {
            title: wire.title,
            item_type: wire.item_type,
            valid: wire.valid,
            subtitle: wire.subtitle,
            arg: wire.arg,
            autocomplete: wire.autocomplete,
            uid: wire.uid,
            icon: wire.icon,
            text: wire.text,
            quick_look_url: wire.quick_look_url,
            match_text: wire.match_text,
            mods: wire
                .mods
                .map(canonicalize_modifier_map)
                .transpose()
                .map_err(serde::de::Error::custom)?,
            action: wire.action,
        })
        .map_err(serde::de::Error::custom)
    }
}

fn canonicalize_modifier_map(mods: ModifierMap) -> Result<ModifierMap> {
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

fn canonical_modifier_key<I>(keys: I) -> Result<String>
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
