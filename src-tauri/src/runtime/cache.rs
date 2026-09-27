//! Small in-memory caches for ZIP and CHM data.
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::{Arc, Mutex, OnceLock};

/// Per-key slots, so work for one key can be serialized without blocking other keys.
pub(crate) type KeyedSlots<T> = OnceLock<Mutex<BTreeMap<String, Arc<Mutex<T>>>>>;

/// Return the slot for `key`, creating it on first use.
///
/// # Errors
///
/// Returns an error when the slot map mutex is poisoned.
pub(crate) fn keyed_slot<T: Default>(
    slots: &KeyedSlots<T>,
    key: &str,
) -> Result<Arc<Mutex<T>>, String> {
    let map = slots.get_or_init(|| Mutex::new(BTreeMap::new()));
    let mut guard = map.lock().map_err(|_| "keyed slot map poisoned".to_string())?;
    Ok(guard.entry(key.to_string()).or_default().clone())
}

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
