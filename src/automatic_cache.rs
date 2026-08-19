use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};

use crate::{Error, Result};

/// Alfred Script Filter automatic cache metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutomaticCache {
    seconds: u64,
    loose_reload: Option<bool>,
}

impl Serialize for AutomaticCache {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map =
            serializer.serialize_map(Some(1 + usize::from(self.loose_reload.is_some())))?;
        map.serialize_entry("seconds", &self.seconds)?;
        if let Some(loose_reload) = self.loose_reload {
            map.serialize_entry("loosereload", &loose_reload)?;
        }
        map.end()
    }
}

impl AutomaticCache {
    /// Minimum accepted cache duration in seconds.
    pub const MIN_SECONDS: u64 = 5;
    /// Maximum accepted cache duration in seconds.
    pub const MAX_SECONDS: u64 = 86_400;

    /// Creates automatic cache metadata with the given duration.
    pub fn try_new(seconds: u64) -> Result<Self> {
        Self::try_with_loose_reload(seconds, None)
    }

    /// Creates automatic cache metadata with an optional `loosereload` flag.
    pub fn try_with_loose_reload(seconds: u64, loose_reload: Option<bool>) -> Result<Self> {
        if !(Self::MIN_SECONDS..=Self::MAX_SECONDS).contains(&seconds) {
            return Err(Error::InvalidAutomaticCacheSeconds {
                seconds,
                min: Self::MIN_SECONDS,
                max: Self::MAX_SECONDS,
            });
        }

        Ok(Self {
            seconds,
            loose_reload,
        })
    }

    /// Returns the cache duration in seconds.
    pub fn seconds(&self) -> u64 {
        self.seconds
    }

    /// Returns the optional Alfred `loosereload` flag.
    pub fn loose_reload(&self) -> Option<bool> {
        self.loose_reload
    }
}

impl<'de> Deserialize<'de> for AutomaticCache {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire {
            seconds: u64,
            #[serde(rename = "loosereload", default)]
            loose_reload: Option<bool>,
        }

        let wire = Wire::deserialize(deserializer)?;
        Self::try_with_loose_reload(wire.seconds, wire.loose_reload)
            .map_err(serde::de::Error::custom)
    }
}
