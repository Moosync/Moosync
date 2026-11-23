use cache::{
    AppCacheEntry, CacheCategory as CoreCacheCategory, CacheReport, ExtensionCacheEntry,
    OrphanCacheEntry, OrphanCacheKind as CoreOrphanKind,
};
use rstest::rstest;

use super::cache::report_to_ui_models;
use crate::{CacheCategory, OrphanCacheKind};

#[rstest]
#[tracing::instrument(level = "debug", skip_all)]
fn test_report_to_ui_models() {
    let report = CacheReport {
        app_caches: vec![AppCacheEntry {
            category: CoreCacheCategory::ImageCache,
            size_bytes: 2048,
            entry_count: 4,
        }],
        extension_caches: vec![ExtensionCacheEntry {
            package_name: "ext.spotify".to_string(),
            display_name: "Spotify".to_string(),
            disk_bytes: 1024,
            db_bytes: 512,
            db_entries: 3,
        }],
        orphan_caches: vec![OrphanCacheEntry {
            kind: CoreOrphanKind::OrphanLyrics,
            size_bytes: 256,
            entry_count: 2,
        }],
        total_bytes: 3840,
    };

    let (total_text, app_items, ext_items, orphan_items) = report_to_ui_models(&report);

    assert_eq!(total_text, "3.75 KiB");
    assert_eq!(app_items.len(), 1);
    assert_eq!(app_items[0].category, CacheCategory::ImageCache);
    assert_eq!(app_items[0].size_text, "2 KiB");
    assert_eq!(app_items[0].entry_count, 4);
    assert_eq!(ext_items.len(), 1);
    assert_eq!(ext_items[0].package_name, "ext.spotify");
    assert_eq!(ext_items[0].size_text, "1.50 KiB");
    assert_eq!(orphan_items.len(), 1);
    assert_eq!(orphan_items[0].kind, OrphanCacheKind::OrphanLyrics);
    assert_eq!(orphan_items[0].size_text, "256 B");
    assert_eq!(orphan_items[0].entry_count, 2);
}
