//! Process-local in-memory caching.
use acorn_core::prelude::HashMap;
use core::hash::Hash;
use std::sync::Mutex;

/// Bounded, process-local cache for reusable values.
#[derive(Debug)]
pub struct Cache<Key, Value> {
    capacity: usize,
    values: Mutex<HashMap<Key, Value>>,
}
impl<Key, Value> Cache<Key, Value>
where
    Key: Eq + Hash,
    Value: Clone,
{
    /// Create a cache with the given maximum entry count.
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            values: Mutex::new(HashMap::new()),
        }
    }
    /// Return a cloned cached value for a key.
    pub fn get(&self, key: &Key) -> Option<Value> {
        self.values.lock().ok().and_then(|values| values.get(key).cloned())
    }
    /// Cache a value and return it.
    pub fn insert(&self, key: Key, value: Value) -> Value {
        if let Ok(mut values) = self.values.lock() {
            if self.capacity > 0 {
                if values.len() >= self.capacity {
                    values.clear();
                }
                values.insert(key, value.clone());
            }
        }
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_returns_inserted_values() {
        let cache = Cache::new(2);
        cache.insert("key", "value");
        assert_eq!(cache.get(&"key"), Some("value"));
    }
    #[test]
    fn test_cache_stays_within_capacity() {
        let cache = Cache::new(1);
        cache.insert("first", "value");
        cache.insert("second", "value");
        assert_eq!(cache.get(&"first"), None);
        assert_eq!(cache.get(&"second"), Some("value"));
    }
}
