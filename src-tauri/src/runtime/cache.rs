//! Small in-memory caches for ZIP and CHM data.
use std::collections::{HashMap, VecDeque};

pub(crate) struct BoundedCache<V> {
    values: HashMap<String, V>,
    oldest_first: VecDeque<String>,
    capacity: usize,
}

impl<V> BoundedCache<V> {
    pub(crate) fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "cache capacity must be positive");
        Self {
            values: HashMap::new(),
            oldest_first: VecDeque::new(),
            capacity,
        }
    }

    pub(crate) fn get(&mut self, key: &str) -> Option<&V> {
        if self.values.contains_key(key) {
            self.oldest_first.retain(|existing| existing != key);
            self.oldest_first.push_back(key.to_string());
        }
        self.values.get(key)
    }

    pub(crate) fn insert(&mut self, key: String, value: V) {
        if self.values.contains_key(&key) {
            self.oldest_first.retain(|existing| existing != &key);
        }
        self.values.insert(key.clone(), value);
        self.oldest_first.push_back(key);
        if self.values.len() > self.capacity {
            if let Some(oldest) = self.oldest_first.pop_front() {
                self.values.remove(&oldest);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BoundedCache;

    #[test]
    fn evicts_least_recently_used_entry() {
        let mut cache = BoundedCache::new(2);
        cache.insert("a".into(), 1);
        cache.insert("b".into(), 2);
        assert_eq!(cache.get("a"), Some(&1));
        cache.insert("c".into(), 3);
        assert_eq!(cache.get("b"), None);
        assert_eq!(cache.get("a"), Some(&1));
        assert_eq!(cache.get("c"), Some(&3));
    }

    #[test]
    fn replacing_entry_keeps_one_position() {
        let mut cache = BoundedCache::new(2);
        cache.insert("a".into(), 1);
        cache.insert("a".into(), 2);
        cache.insert("b".into(), 3);
        cache.insert("c".into(), 4);
        assert_eq!(cache.get("a"), None);
        assert_eq!(cache.get("b"), Some(&3));
    }
}
