use std::time::{SystemTime, UNIX_EPOCH};

use cache::{DiskCacheFilter, DiskCacheRecord, ExtensionCacheStats};
use sqlx::Row;

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

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn upsert_disk_cache(&self, entry: DiskCacheRecord) -> Result<(), DatabaseError> {
        sqlx::query(
            "INSERT INTO disk_cache (path, category, owner, size, created_at, last_accessed)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(path) DO UPDATE SET
                 category = excluded.category,
                 owner = excluded.owner,
                 size = excluded.size,
                 last_accessed = excluded.last_accessed",
        )
        .bind(&entry.path)
        .bind(&entry.category)
        .bind(&entry.owner)
        .bind(entry.size as i64)
        .bind(entry.created_at as i64)
        .bind(entry.last_accessed as i64)
        .execute(&self.pools.write)
        .await
        .map_err(DatabaseError::Query)?;

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn remove_disk_cache(&self, filter: DiskCacheFilter) -> Result<(), DatabaseError> {
        let (sql, value) = match filter {
            DiskCacheFilter::Path(path) => ("DELETE FROM disk_cache WHERE path = ?1", path),
            DiskCacheFilter::Category(category) => {
                ("DELETE FROM disk_cache WHERE category = ?1", category)
            }
            DiskCacheFilter::Owner(owner) => ("DELETE FROM disk_cache WHERE owner = ?1", owner),
        };
        sqlx::query(sql)
            .bind(value)
            .execute(&self.pools.write)
            .await
            .map_err(DatabaseError::Query)?;

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn get_disk_cache_entries(&self) -> Result<Vec<DiskCacheRecord>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT path, category, owner, size, created_at, last_accessed FROM disk_cache",
        )
        .fetch_all(&self.pools.read)
        .await
        .map_err(DatabaseError::Query)?;

        let records = rows
            .into_iter()
            .map(|row| DiskCacheRecord {
                path: row.get::<String, _>("path"),
                category: row.get::<String, _>("category"),
                owner: row.get::<String, _>("owner"),
                size: row.get::<i64, _>("size").max(0) as u64,
                created_at: row.get::<i64, _>("created_at").max(0) as u64,
                last_accessed: row.get::<i64, _>("last_accessed").max(0) as u64,
            })
            .collect();

        Ok(records)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn get_extension_cache_stats(
        &self,
    ) -> Result<Vec<ExtensionCacheStats>, DatabaseError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let rows = sqlx::query(
            "SELECT
                package_name,
                COALESCE(SUM(size), 0) AS total_size_bytes,
                COUNT(*) AS entry_count,
                COALESCE(SUM(CASE WHEN expiry_timestamp <= ?1 THEN 1 ELSE 0 END), 0) AS expired_count,
                COALESCE(SUM(CASE WHEN expiry_timestamp <= ?1 THEN size ELSE 0 END), 0) AS expired_size_bytes
             FROM extension_cache
             GROUP BY package_name",
        )
        .bind(now)
        .fetch_all(&self.pools.read)
        .await
        .map_err(DatabaseError::Query)?;

        let stats = rows
            .into_iter()
            .map(|row| ExtensionCacheStats {
                package_name: row.get::<String, _>("package_name"),
                total_size_bytes: row.get::<i64, _>("total_size_bytes").max(0) as u64,
                entry_count: row.get::<i64, _>("entry_count").max(0) as u64,
                expired_count: row.get::<i64, _>("expired_count").max(0) as u64,
                expired_size_bytes: row.get::<i64, _>("expired_size_bytes").max(0) as u64,
            })
            .collect();

        Ok(stats)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn clear_expired_extension_cache(&self) -> Result<u64, DatabaseError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let res = sqlx::query("DELETE FROM extension_cache WHERE expiry_timestamp <= ?1")
            .bind(now)
            .execute(&self.pools.write)
            .await
            .map_err(DatabaseError::Query)?;

        Ok(res.rows_affected())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn clear_all_extension_cache(&self) -> Result<(), DatabaseError> {
        sqlx::query("DELETE FROM extension_cache")
            .execute(&self.pools.write)
            .await
            .map_err(DatabaseError::Query)?;

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn get_orphan_lyrics_stats(&self) -> Result<(u64, u64), DatabaseError> {
        let row = sqlx::query(
            "SELECT
                COUNT(*) AS cnt,
                COALESCE(SUM(LENGTH(lyrics)), 0) AS total_bytes
             FROM song_lyrics
             WHERE song_id NOT IN (SELECT _id FROM allsongs)",
        )
        .fetch_one(&self.pools.read)
        .await
        .map_err(DatabaseError::Query)?;

        let count = row.get::<i64, _>("cnt").max(0) as u64;
        let bytes = row.get::<i64, _>("total_bytes").max(0) as u64;
        Ok((count, bytes))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn clear_orphan_lyrics(&self) -> Result<u64, DatabaseError> {
        let res =
            sqlx::query("DELETE FROM song_lyrics WHERE song_id NOT IN (SELECT _id FROM allsongs)")
                .execute(&self.pools.write)
                .await
                .map_err(DatabaseError::Query)?;

        Ok(res.rows_affected())
    }
}
