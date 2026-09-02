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

use std::{thread, time::Duration};

use assertables::{assert_err, assert_matches, assert_ok};
use rstest::{fixture, rstest};
use serde::{Deserialize, Serialize};
use tempdir::TempDir;
use tracing_test::traced_test;

use crate::{cache::CacheHolder, error::DatabaseError};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct TestCachePayload {
    name: String,
    score: i32,
    tags: Vec<String>,
}

struct TestCacheContext {
    pub _temp_dir: TempDir,
    pub cache: CacheHolder,
}

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn cache_context() -> TestCacheContext {
    let temp_dir = TempDir::new("moosync_cache_test").expect("failed to create temp dir");
    let db_path = temp_dir.path().join("moosync_cache_test.db");
    let cache = CacheHolder::new(db_path);
    TestCacheContext {
        _temp_dir: temp_dir,
        cache,
    }
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_cache_holder_new_creates_table_and_sets_wal_pragmas(cache_context: TestCacheContext) {
    let TestCacheContext { cache, .. } = cache_context;

    let conn = cache.pool.get().unwrap();
    let journal_mode: String = conn
        .query_row("PRAGMA journal_mode", [], |r| r.get(0))
        .unwrap();

    assert_eq!(journal_mode.to_lowercase(), "wal");
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_cache_holder_set_and_get_valid_entry(cache_context: TestCacheContext) {
    let TestCacheContext { cache, .. } = cache_context;
    let payload = TestCachePayload {
        name: "Song Track Metadata".to_string(),
        score: 42,
        tags: vec!["rock".to_string(), "classic".to_string()],
    };

    assert_ok!(cache.set("https://api.moosync.app/track/123", &payload, 100));
    let retrieved_res: Result<TestCachePayload, _> = cache.get("https://api.moosync.app/track/123");

    assert_ok!(retrieved_res.as_ref());
    assert_eq!(retrieved_res.unwrap(), payload);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_cache_holder_get_non_existent_key_returns_query_error(cache_context: TestCacheContext) {
    let TestCacheContext { cache, .. } = cache_context;

    let res: Result<TestCachePayload, _> = cache.get("https://api.moosync.app/non_existent");

    assert_err!(res.as_ref());
    assert_matches!(res.unwrap_err(), DatabaseError::Query(_));
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_cache_holder_get_expired_entry_returns_cache_expired(cache_context: TestCacheContext) {
    let TestCacheContext { cache, .. } = cache_context;
    let payload = TestCachePayload {
        name: "Expiring Item".to_string(),
        score: 1,
        tags: vec![],
    };

    assert_ok!(cache.set("https://api.moosync.app/expiring", &payload, 0));
    thread::sleep(Duration::from_millis(50));
    let res: Result<TestCachePayload, _> = cache.get("https://api.moosync.app/expiring");

    assert_err!(res.as_ref());
    assert_matches!(res.unwrap_err(), DatabaseError::CacheExpired);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_cache_holder_set_upsert_replaces_existing_entry_and_refreshes_ttl(
    cache_context: TestCacheContext,
) {
    let TestCacheContext { cache, .. } = cache_context;
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

    assert_ok!(cache.set(key, &payload_v1, 60));
    assert_eq!(cache.get::<TestCachePayload>(key).unwrap(), payload_v1);

    assert_ok!(cache.set(key, &payload_v2, 120));
    assert_eq!(cache.get::<TestCachePayload>(key).unwrap(), payload_v2);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_cache_holder_set_and_get_complex_unicode_and_special_keys(cache_context: TestCacheContext) {
    let TestCacheContext { cache, .. } = cache_context;
    let complex_key = "https://example.com/search?query=初音ミク &genre=ポップ&emoji=🎵#section";
    let payload = TestCachePayload {
        name: "日本語の楽曲 🌸".to_string(),
        score: 999,
        tags: vec!["J-Pop".to_string(), "ボーカロイド".to_string()],
    };

    assert_ok!(cache.set(complex_key, &payload, 300));
    let retrieved_res: Result<TestCachePayload, _> = cache.get(complex_key);

    assert_ok!(retrieved_res.as_ref());
    assert_eq!(retrieved_res.unwrap(), payload);
}
