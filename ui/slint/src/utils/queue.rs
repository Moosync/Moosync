use songs_proto::moosync::types::Playlist;
use state_manager::StateManager;

#[tracing::instrument(level = "debug", skip_all)]
pub async fn save_queue(state_manager: &StateManager, name: String, description: String) {
    let player_handler = state_manager.get_player_handler().await;
    let queue = player_handler.get_queue().to_vec();
    if queue.is_empty() {
        return;
    }

    let database = state_manager.get_database().await;
    let trimmed_name = name.trim();
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
        ..Default::default()
    };

    if let Err(e) = database.create_playlist_with_songs(playlist, &queue) {
        tracing::error!("Failed to save queue as playlist: {:?}", e);
    }
}
