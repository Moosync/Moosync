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

use rusqlite::Connection;

use crate::migrations::{run_migration_cache, run_migrations};

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_run_migrations_creates_schema_and_is_idempotent() {
    let mut conn = Connection::open_in_memory().unwrap();

    // First run
    run_migrations(&mut conn);

    // Verify migrations table created
    let migration_count: i64 = conn
        .query_row(
            "SELECT count(*) FROM __diesel_schema_migrations",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(migration_count, 5);

    // Verify core tables exist
    let tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table'")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .filter_map(|res| res.ok())
        .collect();

    assert!(tables.contains(&"allsongs".to_string()));
    assert!(tables.contains(&"albums".to_string()));
    assert!(tables.contains(&"artists".to_string()));
    assert!(tables.contains(&"genres".to_string()));
    assert!(tables.contains(&"playlists".to_string()));
    assert!(tables.contains(&"playlist_bridge".to_string()));
    assert!(tables.contains(&"artist_bridge".to_string()));
    assert!(tables.contains(&"genre_bridge".to_string()));
    assert!(tables.contains(&"analytics".to_string()));

    // Run migrations second time to verify idempotency
    run_migrations(&mut conn);

    let migration_count_second_run: i64 = conn
        .query_row(
            "SELECT count(*) FROM __diesel_schema_migrations",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(migration_count_second_run, 5);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_run_migration_cache_creates_cache_table_and_is_idempotent() {
    let mut conn = Connection::open_in_memory().unwrap();

    run_migration_cache(&mut conn);

    let cache_table_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='cache')",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(cache_table_exists);

    // Run again to verify idempotency
    run_migration_cache(&mut conn);

    let migration_count: i64 = conn
        .query_row(
            "SELECT count(*) FROM __diesel_schema_migrations",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(migration_count, 1);
}
