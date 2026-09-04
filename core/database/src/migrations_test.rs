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
