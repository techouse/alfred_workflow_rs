use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use cached::ConcurrentCached;
use cached::stores::DiskCache;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::{AutomaticCache, Error, Result, Workflow};

/// Generic workflow cache interface.
pub trait WorkflowCache<T> {
    /// Returns the value for a key if present and not expired.
    fn get(&self, key: &str) -> Result<Option<T>>;
    /// Stores a value and returns any previous value.
    fn put(&self, key: &str, value: T) -> Result<Option<T>>;
    /// Removes a value and returns any previous value.
    fn remove(&self, key: &str) -> Result<Option<T>>;
}

/// File-backed cache used by workflow and updater APIs.
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
    /// Default cache name used for query result caches.
    pub const DEFAULT_NAME: &'static str = "query_cache";

    /// Creates a cache at the default path.
    pub fn new() -> Self {
        Self::with_path(Self::default_path())
    }

    /// Creates a cache at a custom directory with default settings.
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

    /// Creates a cache builder at a custom directory.
    pub fn builder(path: impl Into<PathBuf>) -> FileCacheBuilder<T> {
        FileCacheBuilder::new(path)
    }

    /// Creates a cache from explicit settings.
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

    /// Returns the default cache path.
    pub fn default_path() -> PathBuf {
        std::env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(Path::to_path_buf))
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
    }

    /// Hashes a cache key with lower-case MD5 hex, matching the Dart package.
    pub fn hash_key(key: &str) -> String {
        format!("{:x}", md5::compute(key.as_bytes()))
    }

    /// Returns the cache directory.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the cache name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the maximum number of entries tracked for LRU eviction.
    pub fn max_entries(&self) -> usize {
        self.max_entries
    }

    /// Returns the time-to-live in seconds.
    pub fn time_to_live_seconds(&self) -> u64 {
        self.time_to_live_seconds
    }

    /// Returns whether verbose cache mode is enabled.
    pub fn verbose(&self) -> bool {
        self.verbose
    }

    /// Sets the maximum number of entries.
    pub fn set_max_entries(&mut self, max_entries: usize) -> Result<()> {
        validate_max_entries(max_entries)?;
        self.set_max_entries_unchecked(max_entries);
        Ok(())
    }

    /// Sets the time-to-live in seconds.
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

/// Builder for [`FileCache`] configuration.
#[derive(Clone, Debug)]
pub struct FileCacheBuilder<T> {
    path: PathBuf,
    name: String,
    max_entries: usize,
    time_to_live_seconds: u64,
    verbose: bool,
    _value: PhantomData<fn() -> T>,
}

impl<T> FileCacheBuilder<T> {
    /// Creates a builder at the given cache directory.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            name: FileCache::<T>::DEFAULT_NAME.to_owned(),
            max_entries: Workflow::DEFAULT_MAX_CACHE_ENTRIES,
            time_to_live_seconds: Workflow::DEFAULT_CACHE_TIME_TO_LIVE,
            verbose: false,
            _value: PhantomData,
        }
    }

    /// Sets the cache name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Sets the maximum number of entries.
    pub fn max_entries(mut self, max_entries: usize) -> Self {
        self.max_entries = max_entries;
        self
    }

    /// Sets the time-to-live in seconds.
    pub fn time_to_live_seconds(mut self, seconds: u64) -> Self {
        self.time_to_live_seconds = seconds;
        self
    }

    /// Sets verbose cache mode.
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Builds the cache and validates the configuration.
    pub fn build(self) -> Result<FileCache<T>> {
        FileCache::try_with_config(
            self.path,
            self.name,
            self.max_entries,
            self.time_to_live_seconds,
            self.verbose,
        )
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
    /// Returns the value for a key if present and not expired.
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

    /// Stores a value and returns any previous value.
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

    /// Removes a value and returns any previous value.
    pub fn remove(&self, key: &str) -> Result<Option<T>> {
        let cache = self.build_cache()?;
        let previous = cache.cache_remove(&key.to_owned()).map_err(cache_error)?;
        self.forget_key(key)?;
        Ok(previous)
    }

    fn build_cache(&self) -> Result<DiskCache<String, T>> {
        DiskCache::new(&self.name)
            .disk_directory(&self.path)
            .ttl(std::time::Duration::from_secs(self.time_to_live_seconds))
            .sync_to_disk_on_cache_change(true)
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
