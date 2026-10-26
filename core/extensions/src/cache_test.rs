use crate::cache::{ExtensionMemoryCache, MAX_EXTENSION_CACHE_SIZE_BYTES};

#[test]
fn test_lru_strict_boundary() {
    let mut cache = ExtensionMemoryCache::new(50);

    cache.put("k1".to_string(), vec![1; 18], 3600); // 2 + 18 = 20
    cache.put("k2".to_string(), vec![2; 18], 3600); // 2 + 18 = 20 (total 40)
    assert_eq!(cache.size_bytes(), 40);
    assert_eq!(cache.len(), 2);

    cache.put("k3".to_string(), vec![3; 18], 3600); // 2 + 18 = 20 (would be 60 -> evicts k1)
    assert_eq!(cache.size_bytes(), 40);
    assert_eq!(cache.len(), 2);
    assert_eq!(cache.get("k1"), None);
    assert!(cache.get("k2").is_some());
    assert!(cache.get("k3").is_some());
}

#[test]
fn test_lru_key_plus_data_accounting() {
    let mut cache = ExtensionMemoryCache::new(100);
    let key = "long_key_name_that_takes_up_space"; // 33 bytes
    let data = vec![0u8; 17]; // 17 bytes
    cache.put(key.to_string(), data, 3600);
    assert_eq!(cache.size_bytes(), 33 + 17);
}

#[test]
fn test_lru_promotion_on_read() {
    let mut cache = ExtensionMemoryCache::new(50);
    cache.put("k1".to_string(), vec![1; 18], 3600); // 20
    cache.put("k2".to_string(), vec![2; 18], 3600); // 20

    // Access k1 to promote it
    assert!(cache.get("k1").is_some());

    // Insert k3 -> should evict k2 because k1 was promoted
    cache.put("k3".to_string(), vec![3; 18], 3600); // 20
    assert!(cache.get("k1").is_some());
    assert_eq!(cache.get("k2"), None);
    assert!(cache.get("k3").is_some());
}

#[test]
fn test_lru_update_size_adjustment() {
    let mut cache = ExtensionMemoryCache::new(100);
    cache.put("k1".to_string(), vec![1; 20], 3600); // 2 + 20 = 22
    assert_eq!(cache.size_bytes(), 22);

    cache.put("k1".to_string(), vec![1; 40], 3600); // 2 + 40 = 42
    assert_eq!(cache.size_bytes(), 42);

    cache.put("k1".to_string(), vec![1; 10], 3600); // 2 + 10 = 12
    assert_eq!(cache.size_bytes(), 12);
}

#[test]
fn test_oversized_item_bypasses_l1() {
    let mut cache = ExtensionMemoryCache::new(50);
    cache.put("k1".to_string(), vec![1; 18], 3600);
    assert_eq!(cache.len(), 1);

    // Item exceeds max_size_bytes (50)
    cache.put("k_big".to_string(), vec![9; 60], 3600);
    assert_eq!(cache.get("k_big"), None);
    assert_eq!(cache.get("k1"), Some(vec![1; 18]));

    // If an existing key is updated with oversized data, it gets removed from
    // L1
    cache.put("k1".to_string(), vec![9; 60], 3600);
    assert_eq!(cache.get("k1"), None);
    assert_eq!(cache.size_bytes(), 0);
}

#[test]
fn test_l1_ttl_expiration() {
    let mut cache = ExtensionMemoryCache::new(100);
    cache.put("expired".to_string(), vec![1, 2, 3], 0);

    assert_eq!(cache.get("expired"), None);
    assert_eq!(cache.len(), 0);
    assert_eq!(cache.size_bytes(), 0);
}

#[test]
fn test_remove_and_clear() {
    let mut cache = ExtensionMemoryCache::new(100);
    cache.put("k1".to_string(), vec![1; 10], 3600); // 12
    cache.put("k2".to_string(), vec![2; 10], 3600); // 12
    assert_eq!(cache.size_bytes(), 24);

    let removed = cache.remove("k1");
    assert_eq!(removed, Some(vec![1; 10]));
    assert_eq!(cache.size_bytes(), 12);

    cache.clear();
    assert_eq!(cache.size_bytes(), 0);
    assert_eq!(cache.len(), 0);
    assert!(cache.is_empty());
}

#[test]
fn test_default_max_size() {
    let cache = ExtensionMemoryCache::default();
    assert_eq!(cache.max_size_bytes(), MAX_EXTENSION_CACHE_SIZE_BYTES);
    assert_eq!(cache.max_size_bytes(), 1024 * 1024);
}
