use rstest::{fixture, rstest};
use rusqlite::Connection;
use tracing_test::traced_test;

use crate::migrations::run_migrations;

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn memory_connection() -> Connection {
    Connection::open_in_memory().expect("failed to open memory sqlite connection")
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_run_migrations_creates_schema_and_is_idempotent(mut memory_connection: Connection) {
    run_migrations(&mut memory_connection);
    let migration_count: i64 = memory_connection
        .query_row(
            "SELECT count(*) FROM __diesel_schema_migrations",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(migration_count, 5);

    let tables: Vec<String> = memory_connection
        .prepare("SELECT name FROM sqlite_master WHERE type='table'")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .filter_map(|res| res.ok())
        .collect();

    for table in &[
        "allsongs",
        "albums",
        "artists",
        "genres",
        "playlists",
        "playlist_bridge",
        "artist_bridge",
        "genre_bridge",
        "analytics",
    ] {
        assert!(tables.contains(&table.to_string()));
    }

    run_migrations(&mut memory_connection);
    let migration_count_second_run: i64 = memory_connection
        .query_row(
            "SELECT count(*) FROM __diesel_schema_migrations",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(migration_count_second_run, 5);
}
