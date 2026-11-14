use extensions::ExtensionGroupExt;
use extensions_proto::moosync::types::{ExtensionProviderScope, RequestedLyricsRequest};
use futures::StreamExt;
use songs_proto::moosync::types::{Lyrics, Song};
use state_manager::StateManager;
use tracing::Instrument;
use types::prelude::SongsExt;

#[tracing::instrument(level = "debug", skip_all)]
pub async fn fetch_song_lyrics(state_manager: &StateManager, song: &Song) -> Option<Lyrics> {
    let song_id = song.get_id()?;

    let db = state_manager.get_database();
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

    let ext_handler = state_manager.get_extension_handler();
    let lyric_extensions = ext_handler
        .get_extensions_with_scope(ExtensionProviderScope::Lyrics)
        .await;

    let mut stream = lyric_extensions.with(|ext| {
        async move {
            ext.get_lyrics(RequestedLyricsRequest {
                song: Some(song.clone()),
            })
            .await
        }
        .in_current_span()
    });

    while let Some(res) = stream.next().await {
        match res {
            Ok(resp) => {
                if let Some(lyrics) = resp.lyrics {
                    if !lyrics.lines.is_empty() {
                        if let Err(e) = db.update_lyrics(&song_id, &lyrics) {
                            tracing::error!("Failed to cache lyrics in database: {e:?}");
                        }
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
