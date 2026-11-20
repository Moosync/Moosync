use sqlx::SqlitePool;

use crate::error::DatabaseError;

#[tracing::instrument(level = "debug", skip_all)]
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), DatabaseError> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(DatabaseError::Migration)
}
