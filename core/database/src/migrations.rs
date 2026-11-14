#[tracing::instrument(level = "debug", skip_all)]
pub fn run_migrations(conn: &mut rusqlite::Connection) {
    let tx = conn
        .transaction()
        .expect("Failed to start migration transaction");

    tx.execute(
        "CREATE TABLE IF NOT EXISTS __diesel_schema_migrations (
            version TEXT PRIMARY KEY NOT NULL,
            run_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
         )",
        [],
    )
    .expect("Failed to create migrations table");

    let migrations = vec![
        (
            "20240211011442",
            include_str!("../migrations/2024-02-11-011442_initial/up.sql"),
        ),
        (
            "20240211011529",
            include_str!("../migrations/2024-02-11-011529_triggers/up.sql"),
        ),
        (
            "20240211011554",
            include_str!("../migrations/2024-02-11-011554_indices/up.sql"),
        ),
        (
            "20240819175546",
            include_str!("../migrations/2024-08-19-175546_library_item/up.sql"),
        ),
        (
            "20240823072906",
            include_str!("../migrations/2024-08-23-072906_playlist_library_item/up.sql"),
        ),
    ];

    for (version, sql) in migrations {
        let already_run: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM __diesel_schema_migrations WHERE version = ?1)",
                [version],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !already_run {
            tx.execute_batch(sql)
                .unwrap_or_else(|_| panic!("Failed to run migration {}", version));
            tx.execute(
                "INSERT INTO __diesel_schema_migrations (version) VALUES (?1)",
                [version],
            )
            .expect("Failed to log migration");
        }
    }

    tx.commit().expect("Failed to commit migrations");
}
