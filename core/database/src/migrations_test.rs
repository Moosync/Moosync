use rstest::rstest;
use sqlx::SqlitePool;
use tracing_test::traced_test;

use crate::migrations::run_migrations;

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_run_migrations_creates_schema_and_is_idempotent() {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("failed to open memory sqlite pool");

    run_migrations(&pool).await.expect("migrations failed");

    let migration_count: i64 = sqlx::query_scalar("SELECT count(*) FROM _sqlx_migrations")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(migration_count, 2);

    let tables: Vec<String> =
        sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type='table'")
            .fetch_all(&pool)
            .await
            .unwrap();

    for table in &[
        "allsongs",
        "song_lyrics",
        "albums",
        "disk_cache",
        "artists",
        "genres",
        "playlists",
        "playlist_bridge",
        "artist_bridge",
        "genre_bridge",
        "analytics",
        "extension_cache",
        "search_fts",
    ] {
        assert!(tables.contains(&table.to_string()), "missing table {table}");
    }

    run_migrations(&pool)
        .await
        .expect("second migration run failed");
    let migration_count_second_run: i64 =
        sqlx::query_scalar("SELECT count(*) FROM _sqlx_migrations")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(migration_count_second_run, 2);
}
