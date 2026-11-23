use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use tokio::runtime::Handle;
use tracing::Instrument;

use crate::{
    backend::CacheBackend,
    error::CacheError,
    get_backend,
    models::{
        AppCacheEntry, CacheCategory, CacheReport, DiskCacheFilter, DiskCacheRecord,
        ExtensionCacheEntry, ExtensionCacheStats, InstalledExtensionInfo, OrphanCacheEntry,
        OrphanCacheKind,
    },
};

pub struct CacheManager {
    cache_dir: PathBuf,
    backend: Option<Arc<dyn CacheBackend>>,
}

#[tracing::instrument(level = "debug", skip_all)]
fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[tracing::instrument(level = "debug", skip_all)]
fn collect_files_recursive(dir: &Path, out: &mut Vec<(PathBuf, u64)>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files_recursive(&path, out);
        } else if path.is_file()
            && let Ok(meta) = entry.metadata()
        {
            out.push((path, meta.len()));
        }
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn make_record(category: CacheCategory, owner: &str, path: &Path, size: u64) -> DiskCacheRecord {
    let now = now_unix_secs();
    DiskCacheRecord {
        path: path.to_string_lossy().to_string(),
        category: category.as_str().to_string(),
        owner: owner.to_string(),
        size,
        created_at: now,
        last_accessed: now,
    }
}

impl CacheManager {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(cache_dir: PathBuf) -> Self {
        Self {
            cache_dir,
            backend: get_backend(),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn with_backend(cache_dir: PathBuf, backend: Arc<dyn CacheBackend>) -> Self {
        Self {
            cache_dir,
            backend: Some(backend),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn require_backend(&self) -> Result<Arc<dyn CacheBackend>, CacheError> {
        self.backend
            .clone()
            .ok_or(CacheError::BackendNotInitialized)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn write_with<F, E>(
        &self,
        category: CacheCategory,
        owner: &str,
        path: &Path,
        write_fn: F,
    ) -> Result<(), CacheError>
    where
        F: FnOnce(&Path) -> Result<(), E>,
        E: Display,
    {
        if let Some(parent) = path.parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent)?;
        }
        write_fn(path).map_err(|err| CacheError::WriteCallback(err.to_string()))?;
        let record = make_record(category, owner, path, fs::metadata(path)?.len());
        if let Some(backend) = self.backend.clone()
            && let Ok(handle) = Handle::try_current()
        {
            handle.spawn(
                async move {
                    if let Err(err) = backend.upsert_disk_cache(record).await {
                        tracing::error!("Failed to record disk cache entry: {:?}", err);
                    }
                }
                .in_current_span(),
            );
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn write(
        &self,
        category: CacheCategory,
        owner: &str,
        path: &Path,
        contents: &[u8],
    ) -> Result<(), CacheError> {
        self.write_with(category, owner, path, |target| fs::write(target, contents))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn sync_directory(
        &self,
        backend: &Arc<dyn CacheBackend>,
        category: CacheCategory,
        owner: &str,
        dir: &Path,
    ) -> Result<(), CacheError> {
        if !dir.exists() {
            return Ok(());
        }
        let dir_owned = dir.to_path_buf();
        let files = tokio::task::spawn_blocking(move || {
            let mut collected = Vec::new();
            collect_files_recursive(&dir_owned, &mut collected);
            collected
        })
        .await
        .map_err(|err| CacheError::Database(err.to_string()))?;

        for (file_path, size) in files {
            backend
                .upsert_disk_cache(make_record(category, owner, &file_path, size))
                .await?;
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn sync_unmanaged_directories(
        &self,
        backend: &Arc<dyn CacheBackend>,
    ) -> Result<(), CacheError> {
        self.sync_directory(
            backend,
            CacheCategory::Wasmtime,
            "app",
            &self.cache_dir.join("wasmtime").join("cache"),
        )
        .await?;

        let ext_root = self.cache_dir.join("extensions");
        if let Ok(entries) = fs::read_dir(&ext_root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir()
                    && let Some(pkg) = path.file_name().and_then(|n| n.to_str())
                {
                    self.sync_directory(backend, CacheCategory::ExtensionDisk, pkg, &path)
                        .await?;
                }
            }
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn collect_untracked_files(&self, recorded_paths: &HashSet<String>) -> Vec<(PathBuf, u64)> {
        let mut files = Vec::new();
        for dir in [
            self.cache_dir.join("image_cache"),
            self.cache_dir.join("thumbnails"),
        ] {
            collect_files_recursive(&dir, &mut files);
        }
        files
            .into_iter()
            .filter(|(path, _)| !recorded_paths.contains(path.to_string_lossy().as_ref()))
            .collect()
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn get_cache_report(
        &self,
        installed_extensions: &[InstalledExtensionInfo],
    ) -> Result<CacheReport, CacheError> {
        let backend = self.require_backend()?;
        self.sync_unmanaged_directories(&backend).await?;

        let disk_entries = backend.get_disk_cache_entries().await?;
        let ext_db_stats = backend.get_extension_cache_stats().await?;
        let (orphan_lyrics_count, orphan_lyrics_bytes) = backend.get_orphan_lyrics_stats().await?;

        let installed_pkgs: HashSet<&str> = installed_extensions
            .iter()
            .map(|e| e.package_name.as_str())
            .collect();

        let mut app_totals: HashMap<CacheCategory, (u64, u64)> = HashMap::new();
        let mut ext_disk_totals: HashMap<String, (u64, u64)> = HashMap::new();
        let mut stale_records = Vec::new();
        let mut recorded_paths = HashSet::with_capacity(disk_entries.len());

        for entry in disk_entries {
            recorded_paths.insert(entry.path.clone());
            if !Path::new(&entry.path).exists() {
                stale_records.push(entry);
                continue;
            }
            match CacheCategory::from_name(&entry.category) {
                Some(CacheCategory::ExtensionDisk) => {
                    let slot = ext_disk_totals.entry(entry.owner).or_insert((0, 0));
                    slot.0 += entry.size;
                    slot.1 += 1;
                }
                Some(category) => {
                    let slot = app_totals.entry(category).or_insert((0, 0));
                    slot.0 += entry.size;
                    slot.1 += 1;
                }
                None => {}
            }
        }

        let app_caches: Vec<AppCacheEntry> = CacheCategory::app_categories()
            .iter()
            .map(|&category| {
                let (size_bytes, entry_count) =
                    app_totals.get(&category).copied().unwrap_or((0, 0));
                AppCacheEntry {
                    category,
                    size_bytes,
                    entry_count,
                }
            })
            .collect();

        let ext_db_map: HashMap<&str, &ExtensionCacheStats> = ext_db_stats
            .iter()
            .map(|s| (s.package_name.as_str(), s))
            .collect();

        let extension_caches: Vec<ExtensionCacheEntry> = installed_extensions
            .iter()
            .map(|ext| {
                let (disk_bytes, _) = ext_disk_totals
                    .get(&ext.package_name)
                    .copied()
                    .unwrap_or((0, 0));
                let db = ext_db_map.get(ext.package_name.as_str());
                ExtensionCacheEntry {
                    package_name: ext.package_name.clone(),
                    display_name: ext.display_name.clone(),
                    disk_bytes,
                    db_bytes: db.map(|s| s.total_size_bytes).unwrap_or(0),
                    db_entries: db.map(|s| s.entry_count).unwrap_or(0),
                }
            })
            .collect();

        let mut orphan_caches = Vec::new();
        let untracked = self.collect_untracked_files(&recorded_paths);
        if !untracked.is_empty() {
            orphan_caches.push(OrphanCacheEntry {
                kind: OrphanCacheKind::UntrackedDiskFile,
                size_bytes: untracked.iter().map(|(_, s)| *s).sum(),
                entry_count: untracked.len() as u64,
            });
        }
        if !stale_records.is_empty() {
            orphan_caches.push(OrphanCacheEntry {
                kind: OrphanCacheKind::StaleDbRecord,
                size_bytes: stale_records.iter().map(|r| r.size).sum(),
                entry_count: stale_records.len() as u64,
            });
        }
        for (pkg, (size_bytes, entry_count)) in &ext_disk_totals {
            if !installed_pkgs.contains(pkg.as_str()) {
                orphan_caches.push(OrphanCacheEntry {
                    kind: OrphanCacheKind::UninstalledExtensionDisk(pkg.clone()),
                    size_bytes: *size_bytes,
                    entry_count: *entry_count,
                });
            }
        }

        let (mut expired_bytes, mut expired_count) = (0u64, 0u64);
        for stat in &ext_db_stats {
            expired_bytes += stat.expired_size_bytes;
            expired_count += stat.expired_count;
            if !installed_pkgs.contains(stat.package_name.as_str()) {
                orphan_caches.push(OrphanCacheEntry {
                    kind: OrphanCacheKind::UninstalledExtensionDb(stat.package_name.clone()),
                    size_bytes: stat.total_size_bytes,
                    entry_count: stat.entry_count,
                });
            }
        }
        if expired_count > 0 {
            orphan_caches.push(OrphanCacheEntry {
                kind: OrphanCacheKind::ExpiredExtensionDb,
                size_bytes: expired_bytes,
                entry_count: expired_count,
            });
        }
        if orphan_lyrics_count > 0 {
            orphan_caches.push(OrphanCacheEntry {
                kind: OrphanCacheKind::OrphanLyrics,
                size_bytes: orphan_lyrics_bytes,
                entry_count: orphan_lyrics_count,
            });
        }

        let total_bytes = app_caches.iter().map(|c| c.size_bytes).sum::<u64>()
            + extension_caches
                .iter()
                .map(|c| c.disk_bytes + c.db_bytes)
                .sum::<u64>()
            + orphan_caches
                .iter()
                .filter(|o| o.kind != OrphanCacheKind::ExpiredExtensionDb)
                .map(|o| o.size_bytes)
                .sum::<u64>();

        Ok(CacheReport {
            app_caches,
            extension_caches,
            orphan_caches,
            total_bytes,
        })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn clear_app_cache(&self, category: CacheCategory) -> Result<(), CacheError> {
        let backend = self.require_backend()?;
        for entry in backend.get_disk_cache_entries().await? {
            if entry.category == category.as_str() {
                let _ = tokio::fs::remove_file(&entry.path).await;
            }
        }
        if category == CacheCategory::Wasmtime {
            let _ = tokio::fs::remove_dir_all(self.cache_dir.join("wasmtime").join("cache")).await;
        }
        backend
            .remove_disk_cache(DiskCacheFilter::Category(category.as_str().to_string()))
            .await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn clear_extension_cache(&self, package_name: &str) -> Result<(), CacheError> {
        let backend = self.require_backend()?;
        let _ =
            tokio::fs::remove_dir_all(self.cache_dir.join("extensions").join(package_name)).await;
        backend
            .remove_disk_cache(DiskCacheFilter::Owner(package_name.to_string()))
            .await?;
        backend.remove_extension_cache(package_name).await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn clear_orphan_cache(
        &self,
        kind: &OrphanCacheKind,
        installed_extensions: &[InstalledExtensionInfo],
    ) -> Result<(), CacheError> {
        let backend = self.require_backend()?;
        match kind {
            OrphanCacheKind::UntrackedDiskFile => {
                let recorded: HashSet<String> = backend
                    .get_disk_cache_entries()
                    .await?
                    .into_iter()
                    .map(|e| e.path)
                    .collect();
                for (path, _) in self.collect_untracked_files(&recorded) {
                    let _ = tokio::fs::remove_file(&path).await;
                }
            }
            OrphanCacheKind::StaleDbRecord => {
                for entry in backend.get_disk_cache_entries().await? {
                    if !Path::new(&entry.path).exists() {
                        backend
                            .remove_disk_cache(DiskCacheFilter::Path(entry.path))
                            .await?;
                    }
                }
            }
            OrphanCacheKind::ExpiredExtensionDb => {
                backend.clear_expired_extension_cache().await?;
            }
            OrphanCacheKind::OrphanLyrics => {
                backend.clear_orphan_lyrics().await?;
            }
            OrphanCacheKind::UninstalledExtensionDisk(pkg) => {
                if !installed_extensions.iter().any(|e| e.package_name == *pkg) {
                    let _ = tokio::fs::remove_dir_all(self.cache_dir.join("extensions").join(pkg))
                        .await;
                    backend
                        .remove_disk_cache(DiskCacheFilter::Owner(pkg.clone()))
                        .await?;
                }
            }
            OrphanCacheKind::UninstalledExtensionDb(pkg) => {
                if !installed_extensions.iter().any(|e| e.package_name == *pkg) {
                    backend.remove_extension_cache(pkg).await?;
                }
            }
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn clear_all_orphan_caches(
        &self,
        installed_extensions: &[InstalledExtensionInfo],
    ) -> Result<(), CacheError> {
        let report = self.get_cache_report(installed_extensions).await?;
        for orphan in report.orphan_caches {
            self.clear_orphan_cache(&orphan.kind, installed_extensions)
                .await?;
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn clear_all_caches(&self) -> Result<(), CacheError> {
        let backend = self.require_backend()?;
        for &category in CacheCategory::app_categories() {
            self.clear_app_cache(category).await?;
        }
        let _ = tokio::fs::remove_dir_all(self.cache_dir.join("extensions")).await;
        backend
            .remove_disk_cache(DiskCacheFilter::Category(
                CacheCategory::ExtensionDisk.as_str().to_string(),
            ))
            .await?;
        backend.clear_all_extension_cache().await?;
        backend.clear_orphan_lyrics().await?;
        self.clear_orphan_cache(&OrphanCacheKind::UntrackedDiskFile, &[])
            .await?;
        self.clear_orphan_cache(&OrphanCacheKind::StaleDbRecord, &[])
            .await
    }
}
