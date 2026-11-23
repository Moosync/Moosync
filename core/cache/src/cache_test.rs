use std::{
    collections::HashMap,
    fs,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use rstest::rstest;
use tempdir::TempDir;

use crate::{
    backend::CacheBackend,
    error::CacheError,
    manager::CacheManager,
    models::{
        CacheCategory, DiskCacheFilter, DiskCacheRecord, ExtensionCacheStats,
        InstalledExtensionInfo, OrphanCacheKind,
    },
};

#[derive(Default)]
struct MockBackendState {
    disk: HashMap<String, DiskCacheRecord>,
    ext_stats: HashMap<String, ExtensionCacheStats>,
    orphan_lyrics: (u64, u64),
}

#[derive(Default)]
struct MockCacheBackend {
    state: Mutex<MockBackendState>,
}

#[async_trait]
impl CacheBackend for MockCacheBackend {
    #[tracing::instrument(level = "debug", skip_all)]
    async fn upsert_disk_cache(&self, entry: DiskCacheRecord) -> Result<(), CacheError> {
        self.state
            .lock()
            .unwrap()
            .disk
            .insert(entry.path.clone(), entry);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn remove_disk_cache(&self, filter: DiskCacheFilter) -> Result<(), CacheError> {
        let mut guard = self.state.lock().unwrap();
        match filter {
            DiskCacheFilter::Path(path) => {
                guard.disk.remove(&path);
            }
            DiskCacheFilter::Category(category) => {
                guard.disk.retain(|_, v| v.category != category);
            }
            DiskCacheFilter::Owner(owner) => {
                guard.disk.retain(|_, v| v.owner != owner);
            }
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn get_disk_cache_entries(&self) -> Result<Vec<DiskCacheRecord>, CacheError> {
        Ok(self.state.lock().unwrap().disk.values().cloned().collect())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn get_extension_cache_stats(&self) -> Result<Vec<ExtensionCacheStats>, CacheError> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .ext_stats
            .values()
            .cloned()
            .collect())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn remove_extension_cache(&self, package_name: &str) -> Result<(), CacheError> {
        self.state.lock().unwrap().ext_stats.remove(package_name);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn clear_expired_extension_cache(&self) -> Result<u64, CacheError> {
        let mut guard = self.state.lock().unwrap();
        let mut removed = 0;
        for stat in guard.ext_stats.values_mut() {
            removed += stat.expired_count;
            stat.total_size_bytes = stat
                .total_size_bytes
                .saturating_sub(stat.expired_size_bytes);
            stat.entry_count = stat.entry_count.saturating_sub(stat.expired_count);
            stat.expired_count = 0;
            stat.expired_size_bytes = 0;
        }
        Ok(removed)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn clear_all_extension_cache(&self) -> Result<(), CacheError> {
        self.state.lock().unwrap().ext_stats.clear();
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn get_orphan_lyrics_stats(&self) -> Result<(u64, u64), CacheError> {
        Ok(self.state.lock().unwrap().orphan_lyrics)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn clear_orphan_lyrics(&self) -> Result<u64, CacheError> {
        let mut guard = self.state.lock().unwrap();
        let count = guard.orphan_lyrics.0;
        guard.orphan_lyrics = (0, 0);
        Ok(count)
    }
}

#[rstest]
#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_write_and_write_with_records_disk_cache() {
    let tmp = TempDir::new("moosync_cache_test").unwrap();
    let backend = Arc::new(MockCacheBackend::default());
    let manager = CacheManager::with_backend(tmp.path().to_path_buf(), backend.clone());
    let file_path = tmp.path().join("image_cache").join("cover.webp");
    let payload = b"webpdata1234";

    manager
        .write_with(CacheCategory::ImageCache, "app", &file_path, |target| {
            fs::write(target, payload)
        })
        .unwrap();
    tokio::task::yield_now().await;

    let entries = backend.get_disk_cache_entries().await.unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].size, payload.len() as u64);
    assert_eq!(entries[0].category, "image_cache");
    assert_eq!(entries[0].owner, "app");
    assert!(file_path.exists());
}

#[rstest]
#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_report_detects_orphans_and_clears_them() {
    let tmp = TempDir::new("moosync_cache_report_test").unwrap();
    let backend = Arc::new(MockCacheBackend::default());
    let manager = CacheManager::with_backend(tmp.path().to_path_buf(), backend.clone());

    let tracked_img = tmp.path().join("image_cache").join("tracked.webp");
    manager
        .write(CacheCategory::ImageCache, "app", &tracked_img, b"12345")
        .unwrap();
    tokio::task::yield_now().await;

    let untracked_img = tmp.path().join("image_cache").join("untracked.webp");
    fs::write(&untracked_img, b"abcdef").unwrap();

    let uninstalled_ext_file = tmp
        .path()
        .join("extensions")
        .join("old.ext")
        .join("data.bin");
    fs::create_dir_all(uninstalled_ext_file.parent().unwrap()).unwrap();
    fs::write(&uninstalled_ext_file, b"xyz").unwrap();

    backend
        .upsert_disk_cache(DiskCacheRecord {
            path: tmp
                .path()
                .join("image_cache")
                .join("missing.webp")
                .to_string_lossy()
                .to_string(),
            category: "image_cache".to_string(),
            owner: "app".to_string(),
            size: 99,
            created_at: 1,
            last_accessed: 1,
        })
        .await
        .unwrap();

    {
        let mut guard = backend.state.lock().unwrap();
        guard.ext_stats.insert(
            "installed.ext".to_string(),
            ExtensionCacheStats {
                package_name: "installed.ext".to_string(),
                total_size_bytes: 50,
                entry_count: 2,
                expired_count: 1,
                expired_size_bytes: 20,
            },
        );
        guard.ext_stats.insert(
            "old.ext".to_string(),
            ExtensionCacheStats {
                package_name: "old.ext".to_string(),
                total_size_bytes: 30,
                entry_count: 1,
                expired_count: 0,
                expired_size_bytes: 0,
            },
        );
        guard.orphan_lyrics = (3, 120);
    }

    let installed = vec![InstalledExtensionInfo {
        package_name: "installed.ext".to_string(),
        display_name: "Installed Ext".to_string(),
    }];

    let report = manager.get_cache_report(&installed).await.unwrap();

    assert_eq!(report.extension_caches.len(), 1);
    assert_eq!(report.extension_caches[0].db_bytes, 50);
    assert!(
        report
            .orphan_caches
            .iter()
            .any(|o| o.kind == OrphanCacheKind::UntrackedDiskFile)
    );
    assert!(
        report
            .orphan_caches
            .iter()
            .any(|o| o.kind == OrphanCacheKind::StaleDbRecord)
    );
    assert!(
        report
            .orphan_caches
            .iter()
            .any(|o| matches!(o.kind, OrphanCacheKind::UninstalledExtensionDisk(_)))
    );
    assert!(
        report
            .orphan_caches
            .iter()
            .any(|o| matches!(o.kind, OrphanCacheKind::UninstalledExtensionDb(_)))
    );
    assert!(
        report
            .orphan_caches
            .iter()
            .any(|o| o.kind == OrphanCacheKind::ExpiredExtensionDb)
    );
    assert!(
        report
            .orphan_caches
            .iter()
            .any(|o| o.kind == OrphanCacheKind::OrphanLyrics)
    );

    manager.clear_all_orphan_caches(&installed).await.unwrap();
    let after_clear = manager.get_cache_report(&installed).await.unwrap();

    assert!(after_clear.orphan_caches.is_empty());
    assert!(tracked_img.exists());
    assert!(!untracked_img.exists());
    assert!(!uninstalled_ext_file.exists());
}
