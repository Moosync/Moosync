use async_trait::async_trait;

use crate::{
    error::CacheError,
    models::{DiskCacheFilter, DiskCacheRecord, ExtensionCacheStats},
};

#[async_trait]
pub trait CacheBackend: Send + Sync {
    async fn upsert_disk_cache(&self, entry: DiskCacheRecord) -> Result<(), CacheError>;
    async fn remove_disk_cache(&self, filter: DiskCacheFilter) -> Result<(), CacheError>;
    async fn get_disk_cache_entries(&self) -> Result<Vec<DiskCacheRecord>, CacheError>;
    async fn get_extension_cache_stats(&self) -> Result<Vec<ExtensionCacheStats>, CacheError>;
    async fn remove_extension_cache(&self, package_name: &str) -> Result<(), CacheError>;
    async fn clear_expired_extension_cache(&self) -> Result<u64, CacheError>;
    async fn clear_all_extension_cache(&self) -> Result<(), CacheError>;
    async fn get_orphan_lyrics_stats(&self) -> Result<(u64, u64), CacheError>;
    async fn clear_orphan_lyrics(&self) -> Result<u64, CacheError>;
}
