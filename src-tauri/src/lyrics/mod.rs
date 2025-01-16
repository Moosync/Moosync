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

use database::cache::CacheHolder;
use librespot::LibrespotHolder;
use lyrics::LyricsFetcher;
use tauri::State;
use types::errors::Result;

#[tracing::instrument(level = "trace", skip())]
pub fn get_lyrics_state() -> LyricsFetcher {
    LyricsFetcher::new()
}

#[tracing::instrument(
    level = "trace",
    skip(lyrics, librespot, cache, id, url, artists, title)
)]
#[tauri_invoke_proc::parse_tauri_command]
#[tauri::command()]
pub async fn get_lyrics(
    lyrics: State<'_, LyricsFetcher>,
    librespot: State<'_, LibrespotHolder>,
    cache: State<'_, CacheHolder>,
    id: String,
    url: String,
    artists: Vec<String>,
    title: String,
) -> Result<String> {
    let cache_string = format!("get_lyrics_{}_{}_{:?}_{}", id, url, artists, title);

    let cached = cache.get(&cache_string);
    if cached.is_ok() {
        return cached;
    }

    let res = lyrics
        .get_lyrics(librespot.inner(), id, url, artists, title)
        .await?;

    let _ = cache.set(cache_string.as_str(), &res, 7200);
    Ok(res)
}
