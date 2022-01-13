use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use lazy_static::lazy_static;

use crate::dilbert::search::SearchResult;
use crate::dilbert::tags::Tag;

#[derive(Default)]
struct CacheData {
    results: HashMap<SearchResultID, SearchResult>,
    tags: HashMap<Tag, Vec<SearchResultID>>,
}

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
struct SearchResultID(u64);

impl SearchResult {
    fn id(&self) -> SearchResultID {
        let mut h = DefaultHasher::new();
        self.page.hash(&mut h);
        SearchResultID(h.finish())
    }
}

pub struct TagBasedCache {
    data: std::sync::RwLock<CacheData>,
}

lazy_static! {
    pub static ref DILBERT_CACHE: TagBasedCache = TagBasedCache {
        data: std::sync::RwLock::new(CacheData::default())
    };
}

impl TagBasedCache {
    pub fn add(&self, tags: &[Tag], result: &SearchResult) {
        let mut cache_data = self.data.write().unwrap();

        log::info!(
            "Adding an item to cache. Cache contains ${} pages and ${} tags.",
            cache_data.results.len(),
            cache_data.tags.len()
        );

        let result_id = result.id();

        if cache_data.results.contains_key(&result_id) {
            return;
        }

        cache_data.results.insert(result_id, result.clone());

        for tag in tags {
            cache_data
                .tags
                .entry(tag.clone())
                .or_default()
                .push(result_id)
        }
    }

    pub fn find(&self, tags: &[Tag], limit: usize) -> Vec<SearchResult> {
        let cache_data = self.data.read().unwrap();

        let mut occurens = HashMap::<SearchResultID, usize>::default();
        for tag in tags {
            if let Some(results) = cache_data.tags.get(tag) {
                for result_id in results {
                    *occurens.entry(*result_id).or_insert(0) += 1;
                }
            } else {
                continue;
            }
        }

        let mut occurens = occurens
            .into_iter()
            .collect::<Vec<(SearchResultID, usize)>>();
        occurens.sort_unstable_by_key(|(_, count)| *count);

        occurens
            .iter()
            .map(|(id, _)| cache_data.results[id].clone())
            .take(limit)
            .collect()
    }
}
