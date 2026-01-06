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

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub limit: u32,
    pub offset: u32,
    pub token: Option<String>,
    pub is_first: bool,
    pub is_valid: bool,
}

impl Pagination {
    #[tracing::instrument(level = "debug", skip(limit, offset))]
    pub fn new_limit(limit: u32, offset: u32) -> Self {
        Pagination {
            limit,
            offset,
            is_first: true,
            is_valid: true,
            ..Default::default()
        }
    }

    #[tracing::instrument(level = "debug", skip(token))]
    pub fn new_token(token: Option<String>) -> Self {
        Pagination {
            token,
            is_first: true,
            is_valid: true,
            ..Default::default()
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn next_page(&self) -> Self {
        Pagination {
            limit: self.limit,
            offset: self.offset + self.limit.max(1),
            token: self.token.clone(),
            is_first: false,
            is_valid: true,
        }
    }

    #[tracing::instrument(level = "debug", skip(self, token))]
    pub fn next_page_wtoken(&self, token: Option<String>) -> Self {
        Pagination {
            limit: self.limit,
            offset: self.offset + self.limit,
            token,
            is_first: false,
            is_valid: true,
        }
    }

    pub fn invalidate(&mut self) {
        self.is_valid = false;
    }
}
