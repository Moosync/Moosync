use std::time::{SystemTime, UNIX_EPOCH};

use lru::LruCache;

pub const MAX_EXTENSION_CACHE_SIZE_BYTES: usize = 1024 * 1024; // 1 MB

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub data: Vec<u8>,
    pub expiry_timestamp: u64,
}

pub struct ExtensionMemoryCache {
    cache: LruCache<String, CacheEntry>,
    current_size_bytes: usize,
    max_size_bytes: usize,
}

impl Default for ExtensionMemoryCache {
    fn default() -> Self { Self::new(MAX_EXTENSION_CACHE_SIZE_BYTES) }
}

impl ExtensionMemoryCache {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(max_size_bytes: usize) -> Self {
        Self {
            cache: LruCache::unbounded(),
            current_size_bytes: 0,
            max_size_bytes,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get(&mut self, key: &str) -> Option<Vec<u8>> {
        let now_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let entry = self.cache.peek(key)?;
        if now_unix >= entry.expiry_timestamp {
            self.remove(key);
            return None;
        }

        self.cache.get(key).map(|entry| entry.data.clone())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn put(&mut self, key: String, data: Vec<u8>, ttl_seconds: u64) {
        let entry_size = key.len() + data.len();

        if entry_size > self.max_size_bytes {
            self.remove(&key);
            return;
        }

        if let Some(old_entry) = self.cache.pop(&key) {
            self.current_size_bytes -= key.len() + old_entry.data.len();
        }

        let now_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let expiry_timestamp = now_unix.saturating_add(ttl_seconds);

        self.cache.put(
            key,
            CacheEntry {
                data,
                expiry_timestamp,
            },
        );
        self.current_size_bytes += entry_size;

        while self.current_size_bytes > self.max_size_bytes {
            if let Some((evicted_key, evicted_entry)) = self.cache.pop_lru() {
                self.current_size_bytes -= evicted_key.len() + evicted_entry.data.len();
            } else {
                break;
            }
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn remove(&mut self, key: &str) -> Option<Vec<u8>> {
        if let Some(entry) = self.cache.pop(key) {
            self.current_size_bytes -= key.len() + entry.data.len();
            Some(entry.data)
        } else {
            None
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn clear(&mut self) {
        self.cache.clear();
        self.current_size_bytes = 0;
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn len(&self) -> usize { self.cache.len() }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn is_empty(&self) -> bool { self.cache.is_empty() }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn size_bytes(&self) -> usize { self.current_size_bytes }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn max_size_bytes(&self) -> usize { self.max_size_bytes }
}
