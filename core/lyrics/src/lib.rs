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

pub mod error;
use crate::error::LyricsError;

#[cfg(test)]
mod lib_test;
#[cfg(test)]
mod lib_test_smoke;

#[derive(Debug, Clone, Default)]
pub struct LyricsFetcher;

#[plugin_macro::generate]
impl LyricsFetcher {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new() -> Self { Self }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn get_lyrics(
        &self,
        _id: String,
        _url: String,
        _artists: Vec<String>,
        _title: String,
    ) -> Result<String, LyricsError> {
        Ok(String::new())
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn main() {}

impl types::plugin::Plugin for LyricsFetcher {
    #[tracing::instrument(level = "debug", skip_all)]
    fn init(
        _context: &types::plugin::PluginContext,
    ) -> types::plugin::Arc<types::plugin::RwLock<Self>> {
        types::plugin::Arc::new(types::plugin::RwLock::new(LyricsFetcher::new()))
    }
}
