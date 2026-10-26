use tempdir::TempDir;

use crate::Database;

fn create_test_db() -> (Database, TempDir) {
    let tmp = TempDir::new("db_cache_test").expect("Failed to create tempdir");
    let db = Database::new(tmp.path().to_path_buf());
    (db, tmp)
}

#[test]
fn test_cache_set_and_get() {
    let (db, _tmp) = create_test_db();
    let package_name = "test.extension";
    let key = "my_key";
    let data = b"sample_proto_bytes";

    let set_res = db.set_extension_cache(package_name, key, data, 3600);
    assert!(set_res.is_ok());

    let get_res = db.get_extension_cache(package_name, key);
    assert!(get_res.is_ok());
    let opt = get_res.unwrap();
    assert_eq!(opt, Some(data.to_vec()));
}

#[test]
fn test_cache_get_nonexistent() {
    let (db, _tmp) = create_test_db();
    let res = db.get_extension_cache("test.pkg", "nonexistent_key");
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), None);
}

#[test]
fn test_cache_overwrite() {
    let (db, _tmp) = create_test_db();
    let package_name = "test.pkg";
    let key = "key1";

    db.set_extension_cache(package_name, key, b"first_data", 3600)
        .unwrap();
    db.set_extension_cache(package_name, key, b"second_data", 3600)
        .unwrap();

    let res = db.get_extension_cache(package_name, key).unwrap();
    assert_eq!(res, Some(b"second_data".to_vec()));
}

#[test]
fn test_cache_ttl_expiry() {
    let (db, _tmp) = create_test_db();
    let package_name = "test.pkg";
    let key = "expired_key";
    let data = b"will_expire";

    db.set_extension_cache(package_name, key, data, 0).unwrap();

    let res = db.get_extension_cache(package_name, key).unwrap();
    assert_eq!(res, None);

    let res_after = db.get_extension_cache(package_name, key).unwrap();
    assert_eq!(res_after, None);
}

#[test]
fn test_cache_cross_extension_isolation() {
    let (db, _tmp) = create_test_db();
    let key = "common_key";
    let data_a = b"data_from_extension_a";
    let data_b = b"data_from_extension_b";

    db.set_extension_cache("ext.a", key, data_a, 3600).unwrap();
    db.set_extension_cache("ext.b", key, data_b, 3600).unwrap();

    assert_eq!(
        db.get_extension_cache("ext.a", key).unwrap(),
        Some(data_a.to_vec())
    );
    assert_eq!(
        db.get_extension_cache("ext.b", key).unwrap(),
        Some(data_b.to_vec())
    );
}

#[test]
fn test_cache_remove_extension() {
    let (db, _tmp) = create_test_db();
    db.set_extension_cache("ext.remove", "k1", b"v1", 3600)
        .unwrap();
    db.set_extension_cache("ext.remove", "k2", b"v2", 3600)
        .unwrap();
    db.set_extension_cache("ext.keep", "k1", b"v3", 3600)
        .unwrap();

    db.remove_extension_cache("ext.remove").unwrap();

    assert_eq!(db.get_extension_cache("ext.remove", "k1").unwrap(), None);
    assert_eq!(db.get_extension_cache("ext.remove", "k2").unwrap(), None);
    assert_eq!(
        db.get_extension_cache("ext.keep", "k1").unwrap(),
        Some(b"v3".to_vec())
    );
}

#[test]
fn test_cache_binary_safety() {
    let (db, _tmp) = create_test_db();
    let binary_data = vec![0x00, 0xFF, 0x00, 0x12, 0x00, 0x34, 0xFE];
    let key = "key.with.dots/and-dashes";

    db.set_extension_cache("ext.bin", key, &binary_data, 3600)
        .unwrap();
    let result = db.get_extension_cache("ext.bin", key).unwrap();
    assert_eq!(result, Some(binary_data));
}
