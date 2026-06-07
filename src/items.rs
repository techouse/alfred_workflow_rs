use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Deserialize, Serialize};

use crate::{AutomaticCache, Item};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Items {
    items: Vec<Item>,
    exact_order: bool,
    skip_knowledge: Option<bool>,
    cache: Option<AutomaticCache>,
}

impl Items {
    pub fn new(items: Vec<Item>) -> Self {
        Self {
            items,
            exact_order: false,
            skip_knowledge: None,
            cache: None,
        }
    }

    pub fn exact_order(mut self, exact_order: bool) -> Self {
        self.exact_order = exact_order;
        self
    }

    pub fn with_skip_knowledge(mut self, skip_knowledge: bool) -> Self {
        self.skip_knowledge = Some(skip_knowledge);
        self
    }

    pub fn without_skip_knowledge(mut self) -> Self {
        self.skip_knowledge = None;
        self
    }

    pub fn with_cache(mut self, cache: AutomaticCache) -> Self {
        self.cache = Some(cache);
        self
    }

    pub fn without_cache(mut self) -> Self {
        self.cache = None;
        self
    }

    pub fn set_exact_order(&mut self, exact_order: bool) {
        self.exact_order = exact_order;
    }

    pub fn set_skip_knowledge(&mut self, skip_knowledge: Option<bool>) {
        self.skip_knowledge = skip_knowledge;
    }

    pub fn set_cache(&mut self, cache: Option<AutomaticCache>) {
        self.cache = cache;
    }

    pub fn items(&self) -> &[Item] {
        &self.items
    }

    pub fn items_mut(&mut self) -> &mut Vec<Item> {
        &mut self.items
    }

    pub fn exact_order_value(&self) -> bool {
        self.exact_order
    }

    pub fn skip_knowledge(&self) -> Option<bool> {
        self.skip_knowledge
    }

    pub fn cache(&self) -> Option<&AutomaticCache> {
        self.cache.as_ref()
    }

    pub fn push(&mut self, item: Item) {
        self.items.push(item);
    }

    pub fn insert(&mut self, index: usize, item: Item) {
        self.items.insert(index, item);
    }

    pub fn extend<I>(&mut self, items: I)
    where
        I: IntoIterator<Item = Item>,
    {
        self.items.extend(items);
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Item> {
        self.items.iter()
    }
}

impl From<Vec<Item>> for Items {
    fn from(items: Vec<Item>) -> Self {
        Self::new(items)
    }
}

impl IntoIterator for Items {
    type Item = Item;
    type IntoIter = std::vec::IntoIter<Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'a> IntoIterator for &'a Items {
    type Item = &'a Item;
    type IntoIter = std::slice::Iter<'a, Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

impl Serialize for Items {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let len =
            1 + usize::from(self.cache.is_some()) + usize::from(self.skip_knowledge.is_some());
        let mut map = serializer.serialize_map(Some(len))?;
        if let Some(cache) = &self.cache {
            map.serialize_entry("cache", cache)?;
        }
        if let Some(skip_knowledge) = self.skip_knowledge {
            map.serialize_entry("skipknowledge", &skip_knowledge)?;
        }
        map.serialize_entry(
            "items",
            &ItemsSerializer {
                items: &self.items,
                include_uid: !self.exact_order,
            },
        )?;
        map.end()
    }
}

impl<'de> Deserialize<'de> for Items {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire {
            items: Vec<Item>,
            #[serde(rename = "skipknowledge")]
            skip_knowledge: Option<bool>,
            cache: Option<AutomaticCache>,
        }

        let wire = Wire::deserialize(deserializer)?;
        Ok(Self {
            items: wire.items,
            exact_order: false,
            skip_knowledge: wire.skip_knowledge,
            cache: wire.cache,
        })
    }
}

struct ItemsSerializer<'a> {
    items: &'a [Item],
    include_uid: bool,
}

impl Serialize for ItemsSerializer<'_> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.items.len()))?;
        for item in self.items {
            seq.serialize_element(&item.serializer(self.include_uid))?;
        }
        seq.end()
    }
}
