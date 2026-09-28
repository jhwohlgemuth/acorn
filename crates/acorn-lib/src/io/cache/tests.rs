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
