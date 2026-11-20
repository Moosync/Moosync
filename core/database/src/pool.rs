use std::{fs, path::Path, time::Duration};

use sqlx::{
    ConnectOptions, SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
};
use tracing::log::LevelFilter;

use crate::{error::DatabaseError, migrations::run_migrations};

#[derive(Clone)]
pub struct DbPools {
    pub read: SqlitePool,
    pub write: SqlitePool,
}

#[tracing::instrument(level = "debug", skip_all)]
fn build_connect_options(db_path: &Path, read_only: bool) -> SqliteConnectOptions {
    let mut options = SqliteConnectOptions::new()
        .filename(db_path)
        .read_only(read_only)
        .foreign_keys(true)
        .busy_timeout(Duration::from_millis(250))
        .log_statements(LevelFilter::Trace);

    if !read_only {
        options = options
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal);
    }

    options
}

#[tracing::instrument(level = "debug", skip_all)]
async fn remove_legacy_db_if_needed(db_path: &Path) -> Result<(), DatabaseError> {
    if !db_path.exists() {
        return Ok(());
    }

    let check_options = build_connect_options(db_path, true);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(check_options)
        .await
        .map_err(DatabaseError::Connection)?;

    let has_sqlx_migrations: Option<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name = '_sqlx_migrations'",
    )
    .fetch_optional(&pool)
    .await
    .map_err(DatabaseError::Query)?;

    pool.close().await;

    if has_sqlx_migrations.is_none() {
        tracing::info!(
            "Legacy database detected at {:?}; removing and recreating with sqlx migrations",
            db_path
        );
        let base = db_path.to_string_lossy().to_string();
        let _ = fs::remove_file(db_path);
        let _ = fs::remove_file(format!("{base}-wal"));
        let _ = fs::remove_file(format!("{base}-shm"));
    }

    Ok(())
}

#[tracing::instrument(level = "debug", skip_all)]
pub async fn create_pools(db_path: &Path) -> Result<DbPools, DatabaseError> {
    remove_legacy_db_if_needed(db_path).await?;

    let write_options = build_connect_options(db_path, false);
    let write = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(write_options)
        .await
        .map_err(DatabaseError::Connection)?;

    sqlx::raw_sql(
        "PRAGMA wal_autocheckpoint = 1000;
         PRAGMA wal_checkpoint(TRUNCATE);",
    )
    .execute(&write)
    .await
    .map_err(DatabaseError::Query)?;

    run_migrations(&write).await?;

    let read_options = build_connect_options(db_path, true);
    let read = SqlitePoolOptions::new()
        .max_connections(4)
        .connect_with(read_options)
        .await
        .map_err(DatabaseError::Connection)?;

    Ok(DbPools { read, write })
}
