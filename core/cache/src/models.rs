use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CacheCategory {
    ImageCache,
    BlurredCover,
    Thumbnails,
    Wasmtime,
    ExtensionDisk,
    Logs,
}

impl CacheCategory {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ImageCache => "image_cache",
            Self::BlurredCover => "blurred_cover",
            Self::Thumbnails => "thumbnails",
            Self::Wasmtime => "wasmtime",
            Self::ExtensionDisk => "extension_disk",
            Self::Logs => "logs",
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "image_cache" => Some(Self::ImageCache),
            "blurred_cover" => Some(Self::BlurredCover),
            "thumbnails" => Some(Self::Thumbnails),
            "wasmtime" => Some(Self::Wasmtime),
            "extension_disk" => Some(Self::ExtensionDisk),
            "logs" => Some(Self::Logs),
            _ => None,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn app_categories() -> &'static [Self] {
        &[
            Self::ImageCache,
            Self::BlurredCover,
            Self::Thumbnails,
            Self::Wasmtime,
        ]
    }
}

impl fmt::Display for CacheCategory {
    #[tracing::instrument(level = "debug", skip_all)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(self.as_str()) }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskCacheRecord {
    pub path: String,
    pub category: String,
    pub owner: String,
    pub size: u64,
    pub created_at: u64,
    pub last_accessed: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiskCacheFilter {
    Path(String),
    Category(String),
    Owner(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionCacheStats {
    pub package_name: String,
    pub total_size_bytes: u64,
    pub entry_count: u64,
    pub expired_count: u64,
    pub expired_size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledExtensionInfo {
    pub package_name: String,
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppCacheEntry {
    pub category: CacheCategory,
    pub size_bytes: u64,
    pub entry_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionCacheEntry {
    pub package_name: String,
    pub display_name: String,
    pub disk_bytes: u64,
    pub db_bytes: u64,
    pub db_entries: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrphanCacheKind {
    UntrackedDiskFile,
    StaleDbRecord,
    UninstalledExtensionDisk(String),
    UninstalledExtensionDb(String),
    ExpiredExtensionDb,
    OrphanLyrics,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrphanCacheEntry {
    pub kind: OrphanCacheKind,
    pub size_bytes: u64,
    pub entry_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CacheReport {
    pub app_caches: Vec<AppCacheEntry>,
    pub extension_caches: Vec<ExtensionCacheEntry>,
    pub orphan_caches: Vec<OrphanCacheEntry>,
    pub total_bytes: u64,
}
