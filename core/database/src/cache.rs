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

use std::{
    path::PathBuf,
    time::{Duration, SystemTime},
};

use serde::{Deserialize, Serialize};
use tracing::debug;

use super::migrations::run_migration_cache;
use crate::error::DatabaseError;

#[derive(Debug)]
pub struct CacheHolder {
    pool: r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>,
}

impl CacheHolder {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(path: PathBuf) -> Self {
        let db = Self {
            pool: Self::connect(path),
        };

        let mut conn = db.pool.get().expect("Failed to get connection to DB");
        run_migration_cache(&mut conn);
        conn.execute_batch("
            PRAGMA journal_mode = WAL;          -- better write-concurrency
            PRAGMA synchronous = NORMAL;        -- fsync only in critical moments
            PRAGMA wal_autocheckpoint = 1000;   -- write WAL changes back every 1000 pages, for an in average 1MB WAL file. May affect readers if number is increased
            PRAGMA wal_checkpoint(TRUNCATE);    -- free some space by truncating possibly massive WAL files from the last run.
            PRAGMA busy_timeout = 250;          -- sleep if the database is busy
        ").expect("Failed to set DB options");
        db
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn connect(path: PathBuf) -> r2d2::Pool<r2d2_sqlite::SqliteConnectionManager> {
        let manager = r2d2_sqlite::SqliteConnectionManager::file(path).with_init(|conn| {
            conn.trace_v2(
                rusqlite::trace::TraceEventCodes::SQLITE_TRACE_STMT,
                Some(|event| {
                    if let rusqlite::trace::TraceEvent::Stmt(_, sql) = event {
                        tracing::trace!("Executing SQL: {}", sql);
                    }
                }),
            );
            Ok(())
        });

        r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool.")
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set<T>(&self, _url: &str, blob: &T, expires: i32) -> Result<(), DatabaseError>
    where
        T: Serialize,
    {
        let conn = self.pool.get().unwrap();

        let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
        let new_expires = current_time + Duration::from_secs(expires.unsigned_abs() as u64);

        let blob_bytes = serde_json::to_vec(blob)?;
        let expires_secs = new_expires.as_secs() as i64;

        conn.execute(
            "INSERT INTO cache (url, blob, expires) VALUES (?1, ?2, ?3)
             ON CONFLICT(url) DO UPDATE SET
                blob = excluded.blob,
                expires = excluded.expires",
            (_url, &blob_bytes, &expires_secs),
        )
        .map_err(DatabaseError::Query)?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get<T>(&self, _url: &str) -> Result<T, DatabaseError>
    where
        T: for<'a> Deserialize<'a>,
    {
        let conn = self.pool.get().unwrap();

        let (blob, expires): (Vec<u8>, i64) = conn
            .query_row(
                "SELECT blob, expires FROM cache WHERE url = ?1",
                [_url],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(DatabaseError::Query)?;

        let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;

        let expires_dur = Duration::from_secs(expires as u64);
        if current_time > expires_dur {
            debug!("Cache expired for {}", _url);
            return Err(DatabaseError::CacheExpired);
        }

        let parsed: T = serde_json::from_slice(&blob)?;
        Ok(parsed)
    }
}
