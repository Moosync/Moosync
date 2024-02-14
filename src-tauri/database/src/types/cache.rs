use diesel::{AsChangeset, Insertable, Queryable};
use serde::{Deserialize, Serialize};

use crate::cache_schema::cache;

#[derive(Deserialize, Serialize, Insertable, Default, Queryable, AsChangeset, Clone, Debug)]
#[diesel(table_name = cache)]
pub struct CacheModel {
    pub id: Option<i32>,
    pub url: String,
    pub blob: Vec<u8>,
    pub expires: i32,
}
