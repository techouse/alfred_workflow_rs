use std::io::{self, Write};

use crate::{AutomaticCache, FileCache, Item, Items, Result};

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
    pub const DEFAULT_CACHE_TIME_TO_LIVE: u64 = 60;
    pub const DEFAULT_MAX_CACHE_ENTRIES: usize = 10;

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

    pub fn with_automatic_cache(cache: AutomaticCache) -> Self {
        Self {
            automatic_cache: Some(cache),
            use_automatic_cache: true,
            ..Self::new()
        }
    }

    pub fn with_file_cache(file_cache: FileCache<Items>) -> Self {
        Self {
            file_cache,
            ..Self::new()
        }
    }

    pub fn get_items(&self) -> Result<Items> {
        match self.cache_key_hash() {
            Some(cache_key) => Ok(self
                .file_cache
                .get(&cache_key)?
                .unwrap_or_else(|| self.items.clone())),
            None => Ok(self.items.clone()),
        }
    }

    pub fn add_item(&mut self, item: Item) -> Result<()> {
        self.add_item_with_position(item, false)
    }

    pub fn add_item_to_beginning(&mut self, item: Item) -> Result<()> {
        self.add_item_with_position(item, true)
    }

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

    pub fn clear_items(&mut self) -> Result<()> {
        self.items.clear();
        if let Some(cache_key) = self.cache_key_hash() {
            self.file_cache.remove(&cache_key)?;
        }
        Ok(())
    }

    pub fn to_json_string(&self) -> Result<String> {
        self.to_json_string_with(RenderOptions::default())
    }

    pub fn to_json_string_with(&self, options: RenderOptions) -> Result<String> {
        Ok(serde_json::to_string(&self.render_items(options)?)?)
    }

    pub fn write_to<W>(&self, writer: W) -> Result<()>
    where
        W: Write,
    {
        self.write_to_with(writer, RenderOptions::default())
    }

    pub fn write_to_with<W>(&self, mut writer: W, options: RenderOptions) -> Result<()>
    where
        W: Write,
    {
        writer.write_all(self.to_json_string_with(options)?.as_bytes())?;
        Ok(())
    }

    pub fn write_stdout(&self) -> Result<()> {
        self.write_stdout_with(RenderOptions::default())
    }

    pub fn write_stdout_with(&self, options: RenderOptions) -> Result<()> {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        self.write_to_with(&mut handle, options)
    }

    pub fn disable_alfred_smart_result_ordering(&self) -> bool {
        self.disable_alfred_smart_result_ordering
    }

    pub fn set_disable_alfred_smart_result_ordering(&mut self, value: bool) {
        self.disable_alfred_smart_result_ordering = value;
    }

    pub fn skip_knowledge(&self) -> Option<bool> {
        self.skip_knowledge
    }

    pub fn set_skip_knowledge(&mut self, value: Option<bool>) {
        self.skip_knowledge = value;
    }

    pub fn cache_key(&self) -> Option<&str> {
        self.cache_key.as_deref()
    }

    pub fn set_cache_key<S>(&mut self, value: Option<S>)
    where
        S: Into<String>,
    {
        self.cache_key = value.map(Into::into);
        if self.cache_key.is_some() {
            self.use_automatic_cache = false;
        }
    }

    pub fn clear_cache_key(&mut self) {
        self.cache_key = None;
    }

    pub fn cache_key_hash(&self) -> Option<String> {
        self.cache_key.as_deref().map(FileCache::<Items>::hash_key)
    }

    pub fn cache_time_to_live(&self) -> Option<u64> {
        self.cache_time_to_live
    }

    pub fn effective_cache_time_to_live(&self) -> u64 {
        self.cache_time_to_live
            .unwrap_or(Self::DEFAULT_CACHE_TIME_TO_LIVE)
    }

    pub fn set_cache_time_to_live(&mut self, seconds: Option<u64>) {
        self.cache_time_to_live = seconds.filter(|seconds| {
            (AutomaticCache::MIN_SECONDS..=AutomaticCache::MAX_SECONDS).contains(seconds)
        });
        self.file_cache
            .set_time_to_live_seconds_unchecked(self.effective_cache_time_to_live());
    }

    pub fn max_cache_entries(&self) -> Option<usize> {
        self.max_cache_entries
    }

    pub fn effective_max_cache_entries(&self) -> usize {
        self.max_cache_entries
            .unwrap_or(Self::DEFAULT_MAX_CACHE_ENTRIES)
    }

    pub fn set_max_cache_entries(&mut self, entries: Option<usize>) {
        self.max_cache_entries = entries.filter(|entries| *entries > 0);
        if self.max_cache_entries.is_some() {
            self.use_automatic_cache = false;
        }
        self.file_cache
            .set_max_entries_unchecked(self.effective_max_cache_entries());
    }

    pub fn use_automatic_cache(&self) -> bool {
        self.use_automatic_cache
    }

    pub fn set_use_automatic_cache(&mut self, value: bool) {
        if value {
            self.cache_key = None;
        }
        self.use_automatic_cache = value;
    }

    pub fn automatic_cache(&self) -> Option<AutomaticCache> {
        if !self.use_automatic_cache {
            return None;
        }

        self.automatic_cache.clone().or_else(|| {
            AutomaticCache::try_with_loose_reload(self.effective_cache_time_to_live(), Some(true))
                .ok()
        })
    }

    pub fn file_cache(&self) -> &FileCache<Items> {
        &self.file_cache
    }

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

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RenderOptions {
    add_to_beginning: Option<Item>,
    add_to_end: Option<Item>,
}

impl RenderOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_to_beginning(mut self, item: Item) -> Self {
        self.add_to_beginning = Some(item);
        self
    }

    pub fn add_to_end(mut self, item: Item) -> Self {
        self.add_to_end = Some(item);
        self
    }
}
