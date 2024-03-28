#[cfg(feature = "core")]
use diesel::{AsChangeset, Insertable, Queryable};

use serde::{Deserialize, Serialize};

#[cfg(feature = "core")]
use crate::cache_schema::cache;

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
#[cfg_attr(feature = "core", derive(Insertable, Queryable, AsChangeset))]
#[cfg_attr(feature = "core", diesel(table_name = cache))]

pub struct CacheModel {
    pub id: Option<i32>,
    pub url: String,
    pub blob: Vec<u8>,
    pub expires: i32,
}
