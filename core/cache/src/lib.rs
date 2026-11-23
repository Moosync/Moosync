pub mod backend;
#[cfg(test)]
mod cache_test;
pub mod error;
pub mod manager;
pub mod models;

use std::sync::{Arc, OnceLock};

pub use backend::CacheBackend;
pub use error::CacheError;
pub use manager::CacheManager;
pub use models::{
    AppCacheEntry, CacheCategory, CacheReport, DiskCacheFilter, DiskCacheRecord,
    ExtensionCacheEntry, ExtensionCacheStats, InstalledExtensionInfo, OrphanCacheEntry,
    OrphanCacheKind,
};

static GLOBAL_BACKEND: OnceLock<Arc<dyn CacheBackend>> = OnceLock::new();

#[tracing::instrument(level = "debug", skip_all)]
pub fn set_backend(backend: Arc<dyn CacheBackend>) { let _ = GLOBAL_BACKEND.set(backend); }

#[tracing::instrument(level = "debug", skip_all)]
pub fn get_backend() -> Option<Arc<dyn CacheBackend>> { GLOBAL_BACKEND.get().cloned() }
