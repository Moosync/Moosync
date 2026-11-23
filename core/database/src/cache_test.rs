use cache::{CacheBackend, DiskCacheFilter, DiskCacheRecord};
use songs_proto::moosync::types::{LyricLine, Lyrics};
use tempdir::TempDir;

use crate::Database;

async fn create_test_db() -> (Database, TempDir) {
    let tmp = TempDir::new("db_cache_test").expect("Failed to create tempdir");
    let db = Database::new(tmp.path().to_path_buf()).await;
    (db, tmp)
}

#[tokio::test]
async fn test_cache_set_and_get() {
    let (db, _tmp) = create_test_db().await;
    let package_name = "test.extension";
    let key = "my_key";
    let data = b"sample_proto_bytes";

    let set_res = db.set_extension_cache(package_name, key, data, 3600).await;
    assert!(set_res.is_ok());

    let get_res = db.get_extension_cache(package_name, key).await;
    assert!(get_res.is_ok());
    let opt = get_res.unwrap();
    assert_eq!(opt, Some(data.to_vec()));
}

#[tokio::test]
async fn test_cache_get_nonexistent() {
    let (db, _tmp) = create_test_db().await;
    let res = db.get_extension_cache("test.pkg", "nonexistent_key").await;
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), None);
}

#[tokio::test]
async fn test_cache_overwrite() {
    let (db, _tmp) = create_test_db().await;
    let package_name = "test.pkg";
    let key = "key1";

    db.set_extension_cache(package_name, key, b"first_data", 3600)
        .await
        .unwrap();
    db.set_extension_cache(package_name, key, b"second_data", 3600)
        .await
        .unwrap();

    let res = db.get_extension_cache(package_name, key).await.unwrap();
    assert_eq!(res, Some(b"second_data".to_vec()));
}

#[tokio::test]
async fn test_cache_ttl_expiry() {
    let (db, _tmp) = create_test_db().await;
    let package_name = "test.pkg";
    let key = "expired_key";
    let data = b"will_expire";

    db.set_extension_cache(package_name, key, data, 0)
        .await
        .unwrap();

    let res = db.get_extension_cache(package_name, key).await.unwrap();
    assert_eq!(res, None);

    let res_after = db.get_extension_cache(package_name, key).await.unwrap();
    assert_eq!(res_after, None);
}

#[tokio::test]
async fn test_cache_cross_extension_isolation() {
    let (db, _tmp) = create_test_db().await;
    let key = "common_key";
    let data_a = b"data_from_extension_a";
    let data_b = b"data_from_extension_b";

    db.set_extension_cache("ext.a", key, data_a, 3600)
        .await
        .unwrap();
    db.set_extension_cache("ext.b", key, data_b, 3600)
        .await
        .unwrap();

    assert_eq!(
        db.get_extension_cache("ext.a", key).await.unwrap(),
        Some(data_a.to_vec())
    );
    assert_eq!(
        db.get_extension_cache("ext.b", key).await.unwrap(),
        Some(data_b.to_vec())
    );
}

#[tokio::test]
async fn test_cache_remove_extension() {
    let (db, _tmp) = create_test_db().await;
    let pkg = "ext.to_remove";

    db.set_extension_cache(pkg, "k1", b"v1", 3600)
        .await
        .unwrap();
    db.set_extension_cache(pkg, "k2", b"v2", 3600)
        .await
        .unwrap();
    db.set_extension_cache("ext.keep", "k1", b"keep_v1", 3600)
        .await
        .unwrap();

    db.remove_extension_cache(pkg).await.unwrap();

    assert_eq!(db.get_extension_cache(pkg, "k1").await.unwrap(), None);
    assert_eq!(db.get_extension_cache(pkg, "k2").await.unwrap(), None);
    assert_eq!(
        db.get_extension_cache("ext.keep", "k1").await.unwrap(),
        Some(b"keep_v1".to_vec())
    );
}

#[tokio::test]
async fn test_disk_cache_crud_and_filters() {
    let (db, _tmp) = create_test_db().await;
    let record_a = DiskCacheRecord {
        path: "/tmp/a.webp".to_string(),
        category: "image_cache".to_string(),
        owner: "app".to_string(),
        size: 100,
        created_at: 10,
        last_accessed: 10,
    };
    let record_b = DiskCacheRecord {
        path: "/tmp/b.bin".to_string(),
        category: "extension_disk".to_string(),
        owner: "ext.a".to_string(),
        size: 250,
        created_at: 20,
        last_accessed: 20,
    };

    db.upsert_disk_cache(record_a.clone()).await.unwrap();
    db.upsert_disk_cache(record_b.clone()).await.unwrap();
    let entries = db.get_disk_cache_entries().await.unwrap();
    assert_eq!(entries.len(), 2);

    db.remove_disk_cache(DiskCacheFilter::Owner("ext.a".to_string()))
        .await
        .unwrap();
    let remaining = db.get_disk_cache_entries().await.unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].path, "/tmp/a.webp");
}

#[tokio::test]
async fn test_extension_cache_stats_and_expired_cleanup() {
    let (db, _tmp) = create_test_db().await;
    db.set_extension_cache("ext.one", "fresh", b"1234", 3600)
        .await
        .unwrap();
    db.set_extension_cache("ext.one", "stale", b"56789", 0)
        .await
        .unwrap();

    let stats = db.get_extension_cache_stats().await.unwrap();
    assert_eq!(stats.len(), 1);
    assert_eq!(stats[0].package_name, "ext.one");
    assert_eq!(stats[0].entry_count, 2);
    assert_eq!(stats[0].expired_count, 1);

    let cleared = db.clear_expired_extension_cache().await.unwrap();
    assert_eq!(cleared, 1);
    let after = db.get_extension_cache_stats().await.unwrap();
    assert_eq!(after[0].entry_count, 1);
    assert_eq!(after[0].expired_count, 0);
}

#[tokio::test]
async fn test_orphan_lyrics_stats_and_clear() {
    let (db, _tmp) = create_test_db().await;
    let lyrics = Lyrics {
        lines: vec![LyricLine {
            text: "la la la".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };
    db.update_lyrics("missing_song_id", &lyrics).await.unwrap();

    let (count, bytes) = db.get_orphan_lyrics_stats().await.unwrap();
    assert_eq!(count, 1);
    assert!(bytes > 0);

    let cleared = db.clear_orphan_lyrics().await.unwrap();
    assert_eq!(cleared, 1);
    let (after_count, after_bytes) = db.get_orphan_lyrics_stats().await.unwrap();
    assert_eq!(after_count, 0);
    assert_eq!(after_bytes, 0);
}
