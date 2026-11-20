use std::time::{SystemTime, UNIX_EPOCH};

use crate::{error::DatabaseError, pool::DbPools};

#[derive(Clone)]
pub struct CacheRepository {
    pools: DbPools,
}

impl CacheRepository {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(pools: DbPools) -> Self { Self { pools } }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn get_extension_cache(
        &self,
        package_name: &str,
        key: &str,
    ) -> Result<Option<Vec<u8>>, DatabaseError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let row = sqlx::query!(
            "SELECT data, expiry_timestamp FROM extension_cache WHERE package_name = ?1 AND key = ?2",
            package_name,
            key,
        )
        .fetch_optional(&self.pools.read)
        .await
        .map_err(DatabaseError::Query)?;

        if let Some(record) = row {
            if now >= record.expiry_timestamp {
                let _ = sqlx::query!(
                    "DELETE FROM extension_cache WHERE package_name = ?1 AND key = ?2",
                    package_name,
                    key,
                )
                .execute(&self.pools.write)
                .await;
                return Ok(None);
            }
            return Ok(Some(record.data));
        }

        Ok(None)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn set_extension_cache(
        &self,
        package_name: &str,
        key: &str,
        data: &[u8],
        ttl_seconds: u64,
    ) -> Result<(), DatabaseError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let expiry = now.saturating_add(ttl_seconds) as i64;
        let size = (key.len() + data.len()) as i64;

        sqlx::query!(
            "INSERT INTO extension_cache (package_name, key, data, size, expiry_timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(package_name, key) DO UPDATE SET
                 data = excluded.data,
                 size = excluded.size,
                 expiry_timestamp = excluded.expiry_timestamp",
            package_name,
            key,
            data,
            size,
            expiry,
        )
        .execute(&self.pools.write)
        .await
        .map_err(DatabaseError::Query)?;

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn remove_extension_cache(&self, package_name: &str) -> Result<(), DatabaseError> {
        sqlx::query!(
            "DELETE FROM extension_cache WHERE package_name = ?1",
            package_name,
        )
        .execute(&self.pools.write)
        .await
        .map_err(DatabaseError::Query)?;

        Ok(())
    }
}
