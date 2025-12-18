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

use diesel::{
    ExpressionMethods, RunQueryDsl, SqliteConnection,
    connection::SimpleConnection,
    insert_into,
    query_dsl::methods::FilterDsl,
    r2d2::{self, ConnectionManager, Pool},
};

use serde::{Deserialize, Serialize};
use tracing::debug;
use types::errors::{Result, error_helpers};

use super::migrations::run_migration_cache;
use crate::{
    cache_schema::{
        self,
        cache::{dsl::cache, url},
    },
    models::CacheModel,
};

#[derive(Debug)]
pub struct CacheHolder {
    pool: Pool<ConnectionManager<SqliteConnection>>,
}

impl CacheHolder {
    #[tracing::instrument(level = "debug", skip(path))]
    pub fn new(path: PathBuf) -> Self {
        let db = Self {
            pool: Self::connect(path),
        };

        run_migration_cache(&mut db.pool.get().expect("Failed to get connection to DB"));
        db.pool.get().unwrap().batch_execute("
            PRAGMA journal_mode = WAL;          -- better write-concurrency
            PRAGMA synchronous = NORMAL;        -- fsync only in critical moments
            PRAGMA wal_autocheckpoint = 1000;   -- write WAL changes back every 1000 pages, for an in average 1MB WAL file. May affect readers if number is increased
            PRAGMA wal_checkpoint(TRUNCATE);    -- free some space by truncating possibly massive WAL files from the last run.
            PRAGMA busy_timeout = 250;          -- sleep if the database is busy
        ").expect("Failed to set DB options");
        db
    }

    #[tracing::instrument(level = "debug", skip(path))]
    fn connect(path: PathBuf) -> Pool<ConnectionManager<SqliteConnection>> {
        let manager = ConnectionManager::<SqliteConnection>::new(path.to_str().unwrap());

        r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool.")
    }

    #[tracing::instrument(level = "debug", skip(self, _url, blob, expires))]
    pub fn set<T>(&self, _url: &str, blob: &T, expires: i32) -> Result<()>
    where
        T: Serialize,
    {
        let mut conn = self.pool.get().unwrap();

        let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
        let new_expires = current_time + Duration::from_secs(expires.unsigned_abs() as u64);

        let cache_model = CacheModel {
            id: None,
            url: _url.to_string(),
            blob: serde_json::to_vec(blob)?,
            expires: new_expires.as_secs() as i64,
        };
        insert_into(cache)
            .values(&cache_model)
            .on_conflict(cache_schema::cache::url)
            .do_update()
            .set(&cache_model)
            .execute(&mut conn)
            .map_err(error_helpers::to_database_error)?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, _url))]
    pub fn get<T>(&self, _url: &str) -> Result<T>
    where
        T: for<'a> Deserialize<'a>,
    {
        let mut conn = self.pool.get().unwrap();

        let data: CacheModel = cache
            .filter(url.eq(_url))
            .first::<CacheModel>(&mut conn)
            .map_err(error_helpers::to_database_error)?;
        let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;

        let expires = Duration::from_secs(data.expires as u64);
        if current_time > expires {
            debug!("Cache expired for {}", _url);
            return Err("Cache expired".into());
        }

        let parsed: T = serde_json::from_slice(&data.blob)?;
        Ok(parsed)
    }
}
