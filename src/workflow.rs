use std::io::{self, Write};

use crate::{AutomaticCache, FileCache, Item, Items, Result};

/// In-memory Alfred workflow builder and renderer.
///
/// ```
/// use alfred_workflow_rs::{Item, Result, Workflow};
///
/// # fn main() -> Result<()> {
/// let mut workflow = Workflow::builder()
///     .skip_knowledge(Some(true))
///     .build();
/// workflow.add_item(Item::new("Hello"))?;
///
/// assert!(workflow.to_json_string()?.contains(r#""skipknowledge":true"#));
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Workflow {
    items: Items,
    disable_alfred_smart_result_ordering: bool,
    skip_knowledge: Option<bool>,
    automatic_cache: Option<AutomaticCache>,
    use_automatic_cache: bool,
    cache_key: Option<String>,
    cache_time_to_live: Option<u64>,
    max_cache_entries: Option<usize>,
    file_cache: FileCache<Items>,
}

impl Workflow {
    /// Default cache time-to-live used for generated cache configuration.
    pub const DEFAULT_CACHE_TIME_TO_LIVE: u64 = 60;
    /// Default maximum number of cache entries.
    pub const DEFAULT_MAX_CACHE_ENTRIES: usize = 10;

    /// Creates an empty workflow with caching disabled.
    pub fn new() -> Self {
        Self {
            items: Items::default(),
            disable_alfred_smart_result_ordering: false,
            skip_knowledge: None,
            automatic_cache: None,
            use_automatic_cache: false,
            cache_key: None,
            cache_time_to_live: None,
            max_cache_entries: None,
            file_cache: FileCache::default(),
        }
    }

    /// Creates a workflow with custom automatic-cache metadata enabled.
    pub fn with_automatic_cache(cache: AutomaticCache) -> Self {
        Self {
            automatic_cache: Some(cache),
            use_automatic_cache: true,
            ..Self::new()
        }
    }

    /// Creates a workflow with a custom file cache.
    pub fn with_file_cache(file_cache: FileCache<Items>) -> Self {
        Self {
            file_cache,
            ..Self::new()
        }
    }

    /// Creates a workflow builder.
    pub fn builder() -> WorkflowBuilder {
        WorkflowBuilder::new()
    }

    /// Returns current workflow items, reading file cache when a cache key is set.
    pub fn get_items(&self) -> Result<Items> {
        match self.cache_key_hash() {
            Some(cache_key) => Ok(self
                .file_cache
                .get(&cache_key)?
                .unwrap_or_else(|| self.items.clone())),
            None => Ok(self.items.clone()),
        }
    }

    /// Appends an item.
    pub fn add_item(&mut self, item: Item) -> Result<()> {
        self.add_item_with_position(item, false)
    }

    /// Prepends an item.
    pub fn add_item_to_beginning(&mut self, item: Item) -> Result<()> {
        self.add_item_with_position(item, true)
    }

    /// Appends multiple items.
    pub fn add_items<I>(&mut self, items: I) -> Result<()>
    where
        I: IntoIterator<Item = Item>,
    {
        self.items.extend(items);
        if let Some(cache_key) = self.cache_key_hash() {
            self.file_cache.put(&cache_key, self.items.clone())?;
        }
        Ok(())
    }

    /// Clears all workflow items.
    pub fn clear_items(&mut self) -> Result<()> {
        self.items.clear();
        if let Some(cache_key) = self.cache_key_hash() {
            self.file_cache.remove(&cache_key)?;
        }
        Ok(())
    }

    /// Renders the workflow as compact Alfred JSON.
    pub fn to_json_string(&self) -> Result<String> {
        self.to_json_string_with(RenderOptions::default())
    }

    /// Renders the workflow as compact Alfred JSON with temporary render options.
    pub fn to_json_string_with(&self, options: RenderOptions) -> Result<String> {
        Ok(serde_json::to_string(&self.render_items(options)?)?)
    }

    /// Writes rendered workflow JSON to a writer.
    pub fn write_to<W>(&self, writer: W) -> Result<()>
    where
        W: Write,
    {
        self.write_to_with(writer, RenderOptions::default())
    }

    /// Writes rendered workflow JSON to a writer with temporary render options.
    pub fn write_to_with<W>(&self, mut writer: W, options: RenderOptions) -> Result<()>
    where
        W: Write,
    {
        writer.write_all(self.to_json_string_with(options)?.as_bytes())?;
        Ok(())
    }

    /// Writes rendered workflow JSON to stdout.
    pub fn write_stdout(&self) -> Result<()> {
        self.write_stdout_with(RenderOptions::default())
    }

    /// Writes rendered workflow JSON to stdout with temporary render options.
    pub fn write_stdout_with(&self, options: RenderOptions) -> Result<()> {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        self.write_to_with(&mut handle, options)
    }

    /// Returns whether Alfred smart result ordering is disabled.
    pub fn disable_alfred_smart_result_ordering(&self) -> bool {
        self.disable_alfred_smart_result_ordering
    }

    /// Enables or disables Alfred smart result ordering.
    pub fn set_disable_alfred_smart_result_ordering(&mut self, value: bool) {
        self.disable_alfred_smart_result_ordering = value;
    }

    /// Returns the optional Alfred `skipknowledge` flag.
    pub fn skip_knowledge(&self) -> Option<bool> {
        self.skip_knowledge
    }

    /// Sets the optional Alfred `skipknowledge` flag.
    pub fn set_skip_knowledge(&mut self, value: Option<bool>) {
        self.skip_knowledge = value;
    }

    /// Returns the active file-cache key.
    pub fn cache_key(&self) -> Option<&str> {
        self.cache_key.as_deref()
    }

    /// Sets the active file-cache key and disables automatic-cache output.
    pub fn set_cache_key<S>(&mut self, value: Option<S>)
    where
        S: Into<String>,
    {
        self.cache_key = value.map(Into::into);
        if self.cache_key.is_some() {
            self.use_automatic_cache = false;
        }
    }

    /// Clears the active file-cache key.
    pub fn clear_cache_key(&mut self) {
        self.cache_key = None;
    }

    /// Returns the MD5-hashed cache key used on disk.
    pub fn cache_key_hash(&self) -> Option<String> {
        self.cache_key.as_deref().map(FileCache::<Items>::hash_key)
    }

    /// Returns the configured cache time-to-live override.
    pub fn cache_time_to_live(&self) -> Option<u64> {
        self.cache_time_to_live
    }

    /// Returns the effective cache time-to-live in seconds.
    pub fn effective_cache_time_to_live(&self) -> u64 {
        self.cache_time_to_live
            .unwrap_or(Self::DEFAULT_CACHE_TIME_TO_LIVE)
    }

    /// Sets the cache time-to-live override.
    ///
    /// Values outside Alfred's automatic-cache range clear the override.
    pub fn set_cache_time_to_live(&mut self, seconds: Option<u64>) {
        self.cache_time_to_live = seconds.filter(|seconds| {
            (AutomaticCache::MIN_SECONDS..=AutomaticCache::MAX_SECONDS).contains(seconds)
        });
        self.file_cache
            .set_time_to_live_seconds_unchecked(self.effective_cache_time_to_live());
    }

    /// Returns the configured max-cache-entry override.
    pub fn max_cache_entries(&self) -> Option<usize> {
        self.max_cache_entries
    }

    /// Returns the effective max-cache-entry count.
    pub fn effective_max_cache_entries(&self) -> usize {
        self.max_cache_entries
            .unwrap_or(Self::DEFAULT_MAX_CACHE_ENTRIES)
    }

    /// Sets the max-cache-entry override.
    ///
    /// Zero clears the override. Valid values disable automatic-cache output.
    pub fn set_max_cache_entries(&mut self, entries: Option<usize>) {
        self.max_cache_entries = entries.filter(|entries| *entries > 0);
        if self.max_cache_entries.is_some() {
            self.use_automatic_cache = false;
        }
        self.file_cache
            .set_max_entries_unchecked(self.effective_max_cache_entries());
    }

    /// Returns whether automatic-cache output is enabled.
    pub fn use_automatic_cache(&self) -> bool {
        self.use_automatic_cache
    }

    /// Enables or disables automatic-cache output.
    ///
    /// Enabling automatic cache clears any file-cache key.
    pub fn set_use_automatic_cache(&mut self, value: bool) {
        if value {
            self.cache_key = None;
        }
        self.use_automatic_cache = value;
    }

    /// Returns automatic-cache metadata that will be rendered.
    pub fn automatic_cache(&self) -> Option<AutomaticCache> {
        if !self.use_automatic_cache {
            return None;
        }

        self.automatic_cache.clone().or_else(|| {
            AutomaticCache::try_with_loose_reload(self.effective_cache_time_to_live(), Some(true))
                .ok()
        })
    }

    /// Returns the file cache.
    pub fn file_cache(&self) -> &FileCache<Items> {
        &self.file_cache
    }

    /// Replaces the file cache.
    pub fn set_file_cache(&mut self, file_cache: FileCache<Items>) {
        self.file_cache = file_cache;
    }

    fn add_item_with_position(&mut self, item: Item, to_beginning: bool) -> Result<()> {
        let cached_item = item.clone();
        if to_beginning {
            self.items.insert(0, item);
        } else {
            self.items.push(item);
        }

        if let Some(cache_key) = self.cache_key_hash() {
            let mut cached_items = self.file_cache.get(&cache_key)?.unwrap_or_default();
            if to_beginning {
                cached_items.insert(0, cached_item);
            } else {
                cached_items.push(cached_item);
            }
            self.file_cache.put(&cache_key, cached_items)?;
        }

        Ok(())
    }

    fn render_items(&self, options: RenderOptions) -> Result<Items> {
        let mut items = self.get_items()?;
        items.set_exact_order(self.disable_alfred_smart_result_ordering);
        items.set_skip_knowledge(self.skip_knowledge);
        items.set_cache(self.automatic_cache());

        if let Some(item) = options.add_to_beginning {
            items.insert(0, item);
        }
        if let Some(item) = options.add_to_end {
            items.push(item);
        }

        Ok(items)
    }
}

impl Default for Workflow {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for [`Workflow`] configuration.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorkflowBuilder {
    workflow: Workflow,
}

impl WorkflowBuilder {
    /// Creates an empty workflow builder.
    pub fn new() -> Self {
        Self {
            workflow: Workflow::new(),
        }
    }

    /// Sets the initial file cache.
    pub fn file_cache(mut self, file_cache: FileCache<Items>) -> Self {
        self.workflow.set_file_cache(file_cache);
        self
    }

    /// Sets custom automatic-cache metadata and enables automatic-cache output.
    pub fn automatic_cache(mut self, cache: AutomaticCache) -> Self {
        self.workflow.automatic_cache = Some(cache);
        self.workflow.set_use_automatic_cache(true);
        self
    }

    /// Enables or disables automatic-cache output.
    pub fn use_automatic_cache(mut self, value: bool) -> Self {
        self.workflow.set_use_automatic_cache(value);
        self
    }

    /// Enables or disables Alfred smart result ordering.
    pub fn disable_alfred_smart_result_ordering(mut self, value: bool) -> Self {
        self.workflow
            .set_disable_alfred_smart_result_ordering(value);
        self
    }

    /// Sets the optional Alfred `skipknowledge` flag.
    pub fn skip_knowledge(mut self, value: Option<bool>) -> Self {
        self.workflow.set_skip_knowledge(value);
        self
    }

    /// Sets the active file-cache key and disables automatic-cache output.
    pub fn cache_key<S>(mut self, value: Option<S>) -> Self
    where
        S: Into<String>,
    {
        self.workflow.set_cache_key(value);
        self
    }

    /// Sets the cache time-to-live override.
    pub fn cache_time_to_live(mut self, seconds: Option<u64>) -> Self {
        self.workflow.set_cache_time_to_live(seconds);
        self
    }

    /// Sets the max-cache-entry override.
    pub fn max_cache_entries(mut self, entries: Option<usize>) -> Self {
        self.workflow.set_max_cache_entries(entries);
        self
    }

    /// Builds the workflow.
    pub fn build(self) -> Workflow {
        self.workflow
    }
}

/// Temporary render-time item options.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RenderOptions {
    add_to_beginning: Option<Item>,
    add_to_end: Option<Item>,
}

impl RenderOptions {
    /// Creates empty render options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Prepends a temporary item during rendering.
    pub fn add_to_beginning(mut self, item: Item) -> Self {
        self.add_to_beginning = Some(item);
        self
    }

    /// Appends a temporary item during rendering.
    pub fn add_to_end(mut self, item: Item) -> Self {
        self.add_to_end = Some(item);
        self
    }
}
