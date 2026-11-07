use std::path::Path;

use songs_proto::moosync::types::{Playlist, Song};
use state_manager::StateManager;

use super::save_local_image_as_webp;

/// Converts the user-selected cover image into a cached WebP file.
#[tracing::instrument(level = "debug", skip_all)]
fn resolve_cover(state_manager: &StateManager, cover_path: &str) -> Option<String> {
    let trimmed_cover = cover_path.trim();
    if trimmed_cover.is_empty() {
        return None;
    }

    let converted =
        save_local_image_as_webp(Path::new(trimmed_cover), &state_manager.get_cache_dir());
    if converted.is_none() {
        tracing::error!(
            "Failed to convert playlist cover '{}' to webp",
            trimmed_cover
        );
    }
    converted.map(|path| path.to_string_lossy().to_string())
}

#[tracing::instrument(level = "debug", skip_all)]
pub async fn save_queue(
    state_manager: &StateManager,
    name: String,
    description: String,
    cover_path: String,
    songs: Vec<Song>,
) {
    let trimmed_name = name.trim();
    if trimmed_name.is_empty() && songs.is_empty() {
        return;
    }

    let playlist_name = if trimmed_name.is_empty() {
        "Queue playlist".to_string()
    } else {
        trimmed_name.to_string()
    };

    let trimmed_desc = description.trim();
    let playlist_desc = if trimmed_desc.is_empty() {
        None
    } else {
        Some(trimmed_desc.to_string())
    };

    let playlist = Playlist {
        playlist_name,
        playlist_desc,
        playlist_coverpath: resolve_cover(state_manager, &cover_path),
        ..Default::default()
    };

    let database = state_manager.get_database();
    if let Err(e) = database.create_playlist_with_songs(playlist, &songs) {
        tracing::error!("Failed to save playlist: {:?}", e);
    }
}
