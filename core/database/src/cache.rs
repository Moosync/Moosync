use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::OptionalExtension;

use crate::{Database, error::DatabaseError};

impl Database {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn init_cache_table(&self) -> Result<(), DatabaseError> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DatabaseError::Pool(e.to_string()))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS extension_cache (
                package_name TEXT NOT NULL,
                key TEXT NOT NULL,
                data BLOB NOT NULL,
                size INTEGER NOT NULL,
                expiry_timestamp INTEGER NOT NULL,
                PRIMARY KEY (package_name, key)
            );
            CREATE INDEX IF NOT EXISTS idx_cache_expiry ON extension_cache(expiry_timestamp);",
        )
        .map_err(DatabaseError::Query)?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_extension_cache(
        &self,
        package_name: &str,
        key: &str,
    ) -> Result<Option<Vec<u8>>, DatabaseError> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DatabaseError::Pool(e.to_string()))?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let row: Option<(Vec<u8>, i64)> = conn
            .query_row(
                "SELECT data, expiry_timestamp FROM extension_cache WHERE package_name = ?1 AND key = ?2",
                rusqlite::params![package_name, key],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .map_err(DatabaseError::Query)?;

        if let Some((data, expiry)) = row {
            if now >= expiry {
                let _ = conn.execute(
                    "DELETE FROM extension_cache WHERE package_name = ?1 AND key = ?2",
                    rusqlite::params![package_name, key],
                );
                return Ok(None);
            }
            return Ok(Some(data));
        }
        Ok(None)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set_extension_cache(
        &self,
        package_name: &str,
        key: &str,
        data: &[u8],
        ttl_seconds: u64,
    ) -> Result<(), DatabaseError> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DatabaseError::Pool(e.to_string()))?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let expiry = now.saturating_add(ttl_seconds) as i64;
        let size = (key.len() + data.len()) as i64;

        conn.execute(
            "INSERT INTO extension_cache (package_name, key, data, size, expiry_timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(package_name, key) DO UPDATE SET
                 data = excluded.data,
                 size = excluded.size,
                 expiry_timestamp = excluded.expiry_timestamp",
            rusqlite::params![package_name, key, data, size, expiry],
        )
        .map_err(DatabaseError::Query)?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn remove_extension_cache(&self, package_name: &str) -> Result<(), DatabaseError> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DatabaseError::Pool(e.to_string()))?;
        conn.execute(
            "DELETE FROM extension_cache WHERE package_name = ?1",
            rusqlite::params![package_name],
        )
        .map_err(DatabaseError::Query)?;
        Ok(())
    }
}
