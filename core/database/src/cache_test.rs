// Moosync
// Copyright (C) 2024, 2025  Moosync <support@moosync.app>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use std::{env::temp_dir, fs, path::PathBuf, thread, time::Duration};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{cache::CacheHolder, error::DatabaseError};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct TestCachePayload {
    name: String,
    score: i32,
    tags: Vec<String>,
}

#[tracing::instrument(level = "debug", skip_all)]
fn get_test_cache_db_path() -> PathBuf {
    let file_name = format!("moosync_cache_test_{}.db", Uuid::new_v4());
    temp_dir().join(file_name)
}

#[tracing::instrument(level = "debug", skip_all)]
fn cleanup_cache_db(path: &PathBuf) {
    let base_path = path.to_string_lossy().to_string();
    let _ = fs::remove_file(path);
    let _ = fs::remove_file(format!("{}-shm", base_path));
    let _ = fs::remove_file(format!("{}-wal", base_path));
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_cache_holder_new_creates_table_and_sets_wal_pragmas() {
    let db_path = get_test_cache_db_path();
    let cache = CacheHolder::new(db_path.clone());
    let conn = cache.pool.get().unwrap();
    let journal_mode: String = conn
        .query_row("PRAGMA journal_mode", [], |r| r.get(0))
        .unwrap();
    assert_eq!(journal_mode.to_lowercase(), "wal");

    cleanup_cache_db(&db_path);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_cache_holder_set_and_get_valid_entry() {
    let db_path = get_test_cache_db_path();
    let cache = CacheHolder::new(db_path.clone());

    let payload = TestCachePayload {
        name: "Song Track Metadata".to_string(),
        score: 42,
        tags: vec!["rock".to_string(), "classic".to_string()],
    };

    cache
        .set("https://api.moosync.app/track/123", &payload, 100)
        .unwrap();

    let retrieved: TestCachePayload = cache
        .get("https://api.moosync.app/track/123")
        .expect("Cache entry should be retrieved");
    assert_eq!(retrieved, payload);

    cleanup_cache_db(&db_path);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_cache_holder_get_non_existent_key_returns_query_error() {
    let db_path = get_test_cache_db_path();
    let cache = CacheHolder::new(db_path.clone());

    let res: Result<TestCachePayload, _> = cache.get("https://api.moosync.app/non_existent");
    assert!(res.is_err());
    let err = res.unwrap_err();
    match err {
        DatabaseError::Query(_) => {}
        _ => panic!("Expected DatabaseError::Query, got: {:?}", err),
    }

    cleanup_cache_db(&db_path);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_cache_holder_get_expired_entry_returns_cache_expired() {
    let db_path = get_test_cache_db_path();
    let cache = CacheHolder::new(db_path.clone());

    let payload = TestCachePayload {
        name: "Expiring Item".to_string(),
        score: 1,
        tags: vec![],
    };

    cache
        .set("https://api.moosync.app/expiring", &payload, 0)
        .unwrap();

    thread::sleep(Duration::from_millis(50));

    let res: Result<TestCachePayload, _> = cache.get("https://api.moosync.app/expiring");
    assert!(res.is_err());
    let err = res.unwrap_err();
    match err {
        DatabaseError::CacheExpired => {}
        _ => panic!("Expected DatabaseError::CacheExpired, got: {:?}", err),
    }

    cleanup_cache_db(&db_path);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_cache_holder_set_upsert_replaces_existing_entry_and_refreshes_ttl() {
    let db_path = get_test_cache_db_path();
    let cache = CacheHolder::new(db_path.clone());

    let payload_v1 = TestCachePayload {
        name: "Version 1".to_string(),
        score: 10,
        tags: vec!["v1".to_string()],
    };

    let payload_v2 = TestCachePayload {
        name: "Version 2".to_string(),
        score: 20,
        tags: vec!["v2".to_string(), "updated".to_string()],
    };

    let key = "https://api.moosync.app/versioned";

    cache.set(key, &payload_v1, 60).unwrap();
    let res_v1: TestCachePayload = cache.get(key).unwrap();
    assert_eq!(res_v1, payload_v1);

    cache.set(key, &payload_v2, 120).unwrap();
    let res_v2: TestCachePayload = cache.get(key).unwrap();
    assert_eq!(res_v2, payload_v2);

    cleanup_cache_db(&db_path);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_cache_holder_set_and_get_complex_unicode_and_special_keys() {
    let db_path = get_test_cache_db_path();
    let cache = CacheHolder::new(db_path.clone());

    let complex_key = "https://example.com/search?query=初音ミク &genre=ポップ&emoji=🎵#section";
    let payload = TestCachePayload {
        name: "日本語の楽曲 🌸".to_string(),
        score: 999,
        tags: vec!["J-Pop".to_string(), "ボーカロイド".to_string()],
    };

    cache.set(complex_key, &payload, 300).unwrap();

    let retrieved: TestCachePayload = cache.get(complex_key).unwrap();
    assert_eq!(retrieved, payload);

    cleanup_cache_db(&db_path);
}
