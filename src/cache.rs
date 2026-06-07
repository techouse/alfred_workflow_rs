use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use cached::IOCached;
use cached::stores::DiskCache;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::{AutomaticCache, Error, Result, Workflow};

pub trait WorkflowCache<T> {
    fn get(&self, key: &str) -> Result<Option<T>>;
    fn put(&self, key: &str, value: T) -> Result<Option<T>>;
    fn remove(&self, key: &str) -> Result<Option<T>>;
}

#[derive(Clone, Debug)]
pub struct FileCache<T> {
    path: PathBuf,
    name: String,
    max_entries: usize,
    time_to_live_seconds: u64,
    verbose: bool,
    _value: PhantomData<fn() -> T>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct CacheMetadata {
    keys: Vec<String>,
}

impl<T> FileCache<T> {
    pub const DEFAULT_NAME: &'static str = "query_cache";

    pub fn new() -> Self {
        Self::with_path(Self::default_path())
    }

    pub fn with_path(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            name: Self::DEFAULT_NAME.to_owned(),
            max_entries: Workflow::DEFAULT_MAX_CACHE_ENTRIES,
            time_to_live_seconds: Workflow::DEFAULT_CACHE_TIME_TO_LIVE,
            verbose: false,
            _value: PhantomData,
        }
    }

    pub fn try_with_config(
        path: impl Into<PathBuf>,
        name: impl Into<String>,
        max_entries: usize,
        time_to_live_seconds: u64,
        verbose: bool,
    ) -> Result<Self> {
        validate_max_entries(max_entries)?;
        validate_time_to_live(time_to_live_seconds)?;

        Ok(Self {
            path: path.into(),
            name: name.into(),
            max_entries,
            time_to_live_seconds,
            verbose,
            _value: PhantomData,
        })
    }

    pub fn default_path() -> PathBuf {
        std::env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(Path::to_path_buf))
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
    }

    pub fn hash_key(key: &str) -> String {
        format!("{:x}", md5::compute(key.as_bytes()))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn max_entries(&self) -> usize {
        self.max_entries
    }

    pub fn time_to_live_seconds(&self) -> u64 {
        self.time_to_live_seconds
    }

    pub fn verbose(&self) -> bool {
        self.verbose
    }

    pub fn set_max_entries(&mut self, max_entries: usize) -> Result<()> {
        validate_max_entries(max_entries)?;
        self.set_max_entries_unchecked(max_entries);
        Ok(())
    }

    pub fn set_time_to_live_seconds(&mut self, seconds: u64) -> Result<()> {
        validate_time_to_live(seconds)?;
        self.set_time_to_live_seconds_unchecked(seconds);
        Ok(())
    }

    pub(crate) fn set_max_entries_unchecked(&mut self, max_entries: usize) {
        self.max_entries = max_entries;
    }

    pub(crate) fn set_time_to_live_seconds_unchecked(&mut self, seconds: u64) {
        self.time_to_live_seconds = seconds;
    }

    pub(crate) fn with_config_unchecked(
        path: impl Into<PathBuf>,
        name: impl Into<String>,
        max_entries: usize,
        time_to_live_seconds: u64,
        verbose: bool,
    ) -> Self {
        Self {
            path: path.into(),
            name: name.into(),
            max_entries,
            time_to_live_seconds,
            verbose,
            _value: PhantomData,
        }
    }
}

impl<T> Default for FileCache<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> PartialEq for FileCache<T> {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
            && self.name == other.name
            && self.max_entries == other.max_entries
            && self.time_to_live_seconds == other.time_to_live_seconds
            && self.verbose == other.verbose
    }
}

impl<T> Eq for FileCache<T> {}

impl<T> WorkflowCache<T> for FileCache<T>
where
    T: Serialize + DeserializeOwned,
{
    fn get(&self, key: &str) -> Result<Option<T>> {
        FileCache::get(self, key)
    }

    fn put(&self, key: &str, value: T) -> Result<Option<T>> {
        FileCache::put(self, key, value)
    }

    fn remove(&self, key: &str) -> Result<Option<T>> {
        FileCache::remove(self, key)
    }
}

impl<T> FileCache<T>
where
    T: Serialize + DeserializeOwned,
{
    pub fn get(&self, key: &str) -> Result<Option<T>> {
        let cache = self.build_cache()?;
        let value = cache.cache_get(&key.to_owned()).map_err(cache_error)?;

        if value.is_some() {
            self.touch_key(key)?;
        } else {
            self.forget_key(key)?;
        }

        Ok(value)
    }

    pub fn put(&self, key: &str, value: T) -> Result<Option<T>> {
        let cache = self.build_cache()?;
        let previous = cache
            .cache_set(key.to_owned(), value)
            .map_err(cache_error)?;

        for evicted_key in self.record_key_and_evictions(key)? {
            cache.cache_remove(&evicted_key).map_err(cache_error)?;
        }

        Ok(previous)
    }

    pub fn remove(&self, key: &str) -> Result<Option<T>> {
        let cache = self.build_cache()?;
        let previous = cache.cache_remove(&key.to_owned()).map_err(cache_error)?;
        self.forget_key(key)?;
        Ok(previous)
    }

    fn build_cache(&self) -> Result<DiskCache<String, T>> {
        DiskCache::new(&self.name)
            .set_disk_directory(&self.path)
            .set_lifespan(self.time_to_live_seconds)
            .set_sync_to_disk_on_cache_change(true)
            .build()
            .map_err(cache_error)
    }
}

impl<T> FileCache<T> {
    fn metadata_path(&self) -> PathBuf {
        self.path.join(format!("{}_keys.json", self.name))
    }

    fn read_metadata(&self) -> Result<CacheMetadata> {
        match std::fs::read_to_string(self.metadata_path()) {
            Ok(raw) if raw.trim().is_empty() => Ok(CacheMetadata::default()),
            Ok(raw) => serde_json::from_str(&raw).map_err(cache_error),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                Ok(CacheMetadata::default())
            }
            Err(error) => Err(cache_error(error)),
        }
    }

    fn write_metadata(&self, metadata: &CacheMetadata) -> Result<()> {
        std::fs::create_dir_all(&self.path).map_err(cache_error)?;
        let raw = serde_json::to_string(metadata).map_err(cache_error)?;
        std::fs::write(self.metadata_path(), raw).map_err(cache_error)
    }

    fn touch_key(&self, key: &str) -> Result<()> {
        let mut metadata = self.read_metadata()?;
        metadata.keys.retain(|cached_key| cached_key != key);
        metadata.keys.push(key.to_owned());
        self.write_metadata(&metadata)
    }

    fn record_key_and_evictions(&self, key: &str) -> Result<Vec<String>> {
        let mut metadata = self.read_metadata()?;
        metadata.keys.retain(|cached_key| cached_key != key);
        metadata.keys.push(key.to_owned());

        let eviction_count = metadata.keys.len().saturating_sub(self.max_entries);
        let evicted = metadata.keys.drain(0..eviction_count).collect();

        self.write_metadata(&metadata)?;

        Ok(evicted)
    }

    fn forget_key(&self, key: &str) -> Result<()> {
        let mut metadata = self.read_metadata()?;
        let old_len = metadata.keys.len();
        metadata.keys.retain(|cached_key| cached_key != key);

        if metadata.keys.len() != old_len {
            self.write_metadata(&metadata)?;
        }

        Ok(())
    }
}

fn validate_max_entries(entries: usize) -> Result<()> {
    if entries == 0 {
        return Err(Error::InvalidMaxCacheEntries { entries });
    }

    Ok(())
}

fn validate_time_to_live(seconds: u64) -> Result<()> {
    AutomaticCache::try_new(seconds).map(|_| ())
}

fn cache_error(error: impl std::fmt::Display) -> Error {
    Error::Cache(error.to_string())
}
