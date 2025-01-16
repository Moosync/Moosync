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

// Moosync
// Copyright (C) 2025 Moosync
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
// along with this program. If not, see <http://www.gnu.org/licenses/>.

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
    pub expires: i64,
}
