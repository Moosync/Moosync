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
