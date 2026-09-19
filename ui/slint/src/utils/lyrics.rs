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

use extensions_proto::moosync::types::{ExtensionProviderScope, RequestedLyricsRequest};
use songs_proto::moosync::types::{Lyrics, Song};
use state_manager::StateManager;
use types::prelude::SongsExt;

#[tracing::instrument(level = "debug", skip_all)]
pub async fn fetch_song_lyrics(state_manager: &StateManager, song: &Song) -> Option<Lyrics> {
    let song_id = song.get_id()?;

    let db = state_manager.get_database().await;
    match db.get_lyrics(&song_id) {
        Ok(Some(lyrics)) => {
            if !lyrics.lines.is_empty() {
                return Some(lyrics);
            }
        }
        Ok(None) => {}
        Err(e) => {
            tracing::error!("Database query for lyrics failed: {e:?}");
        }
    }

    let ext_handler = state_manager.get_extension_handler().await;
    let lyric_extensions = ext_handler
        .get_extensions_with_scope(ExtensionProviderScope::Lyrics)
        .await;

    for ext in lyric_extensions {
        let req = RequestedLyricsRequest {
            song: Some(song.clone()),
        };
        match ext.get_lyrics(req).await {
            Ok(resp) => {
                if let Some(lyrics) = resp.lyrics {
                    if !lyrics.lines.is_empty() {
                        return Some(lyrics);
                    }
                }
            }
            Err(e) => {
                tracing::error!("Extension get_lyrics request failed: {e:?}");
            }
        }
    }

    None
}
