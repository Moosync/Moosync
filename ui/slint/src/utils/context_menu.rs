use std::rc::Rc;

use slint::{ComponentHandle, Image, Model, ModelRc, SharedString, VecModel, Weak};
use songs_proto::moosync::types::{
    Album, Artist, GetEntityOptions, GetSongOptions, Playlist, Song, entity_result,
};
use state_manager::StateManager;
use tracing::Instrument;
use types::prelude::SongsExt;

use super::{
    lazy_model::make_lazy_song_model,
    models::IntoVec,
    navigation::{goto_album, goto_artist},
};
use crate::{
    AlbumModel, ArtistModel, ContextMenuItem, ContextMenuItems, ContextSubMenuItem, MainWindow,
    Pages, PlaylistContentPageProps, PlaylistsPageProps, SongModel,
};

#[tracing::instrument(level = "debug", skip_all)]
fn make_context_menu_item(
    action_id: impl Into<SharedString>,
    title: impl Into<SharedString>,
    icon: Image,
) -> ContextMenuItem {
    ContextMenuItem {
        action_id: action_id.into(),
        title: title.into(),
        icon,
        has_sub_menu: false,
        sub_items: ModelRc::default(),
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn make_context_sub_item(
    action_id: impl Into<SharedString>,
    title: impl Into<SharedString>,
    icon: Image,
) -> ContextSubMenuItem {
    ContextSubMenuItem {
        action_id: action_id.into(),
        title: title.into(),
        icon,
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn make_context_submenu_item(
    action_id: impl Into<SharedString>,
    title: impl Into<SharedString>,
    icon: Image,
    sub_items: Vec<ContextSubMenuItem>,
) -> ContextMenuItem {
    ContextMenuItem {
        action_id: action_id.into(),
        title: title.into(),
        icon,
        has_sub_menu: true,
        sub_items: ModelRc::new(VecModel::from(sub_items)),
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn populate_navigation_items(
    items: &mut Vec<ContextMenuItem>,
    main_window: &MainWindow,
    song_models: &ModelRc<SongModel>,
) {
    let Some(first_song) = song_models.row_data(0) else {
        return;
    };

    if !first_song.album_name.is_empty() {
        let title = main_window
            .global::<ContextMenuItems>()
            .invoke_get_goto_album_title(first_song.album_name.clone());
        items.push(make_context_menu_item(
            format!("goto_album:{}", first_song.album_id),
            title,
            Image::default(),
        ));
    }

    let artist_sub_items: Vec<ContextSubMenuItem> = (0..first_song.artists.row_count())
        .filter_map(|i| first_song.artists.row_data(i))
        .map(|a| {
            let action_id = format!("goto_artist:{}", a.id);
            make_context_sub_item(action_id, a.title, Image::default())
        })
        .collect();

    if !artist_sub_items.is_empty() {
        let title = main_window
            .global::<ContextMenuItems>()
            .invoke_get_goto_artists_title();
        items.push(make_context_submenu_item(
            "goto_artist",
            title,
            Image::default(),
            artist_sub_items,
        ));
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn attach_playlist_submenu(
    vec_model: Rc<VecModel<ContextMenuItem>>,
    main_window_weak: Weak<MainWindow>,
    state_manager: StateManager,
) {
    let _ = slint::spawn_local(
        async move {
            let db = state_manager.get_database().await;
            let res = match db.get_entity_by_options(GetEntityOptions {
                playlist: Some(Playlist::default()),
                ..Default::default()
            }) {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!("Failed to fetch playlists for context menu: {:?}", e);
                    return;
                }
            };
            let Some(entity_result::Result::Playlists(list)) = res.result else {
                return;
            };
            if list.playlists.is_empty() {
                return;
            }

            let playlist_sub_items: Vec<ContextSubMenuItem> = list
                .playlists
                .into_iter()
                .map(|p| {
                    let pid = p.playlist_id.unwrap_or_default();
                    let action_id = format!("add_to_playlist:{}", pid);
                    make_context_sub_item(action_id, p.playlist_name, Image::default())
                })
                .collect();

            let Some(window) = main_window_weak.upgrade() else {
                return;
            };
            let title = window
                .global::<ContextMenuItems>()
                .invoke_get_add_to_playlist_title();
            let item = make_context_submenu_item(
                "add_to_playlist",
                title,
                Image::default(),
                playlist_sub_items,
            );
            let mut insert_idx = vec_model.row_count();
            for i in 0..vec_model.row_count() {
                if let Some(existing) = vec_model.row_data(i) {
                    if existing.action_id.starts_with("goto_") {
                        insert_idx = i;
                        break;
                    }
                }
            }
            vec_model.insert(insert_idx, item);
        }
        .in_current_span(),
    );
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn build_song_context_menu_items(
    main_window: &MainWindow,
    state_manager: &StateManager,
    song_models: &ModelRc<SongModel>,
) -> ModelRc<ContextMenuItem> {
    let mut all_items: Vec<ContextMenuItem> = main_window
        .global::<ContextMenuItems>()
        .invoke_get_all_songs_items()
        .into_vec();

    if main_window.get_active_page() == Pages::PlaylistContent {
        let remove_item = ContextMenuItem {
            action_id: "remove_from_playlist".into(),
            title: main_window
                .global::<ContextMenuItems>()
                .invoke_get_remove_from_playlist_title(),
            icon: Image::default(),
            has_sub_menu: false,
            sub_items: ModelRc::default(),
        };
        all_items.push(remove_item);
    }

    populate_navigation_items(&mut all_items, main_window, song_models);

    let vec_model = Rc::new(VecModel::from(all_items));
    let model_rc = ModelRc::new(vec_model.clone());

    attach_playlist_submenu(vec_model, main_window.as_weak(), state_manager.clone());

    model_rc
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn build_queue_context_menu_items(
    main_window: &MainWindow,
    state_manager: &StateManager,
    song_models: &ModelRc<SongModel>,
    _idx: i32,
) -> ModelRc<ContextMenuItem> {
    let mut all_items: Vec<ContextMenuItem> = main_window
        .global::<ContextMenuItems>()
        .invoke_get_queue_items()
        .into_vec();

    populate_navigation_items(&mut all_items, main_window, song_models);

    let vec_model = Rc::new(VecModel::from(all_items));
    let model_rc = ModelRc::new(vec_model.clone());

    attach_playlist_submenu(vec_model, main_window.as_weak(), state_manager.clone());

    model_rc
}

#[tracing::instrument(level = "debug", skip_all)]
async fn handle_playback_action(state_manager: &StateManager, songs: Vec<Song>, action: &str) {
    tracing::debug!(
        "Handling playback action '{}' on {} songs",
        action,
        songs.len()
    );
    let mut player = state_manager.get_player_handler_mut().await;
    match action {
        "play_now" => player.play_now(songs),
        "play_next" => player.play_next(songs),
        "clear_queue_and_play" => player.clear_and_play(songs),
        "add_to_queue" => player.add_to_queue(songs),
        _ => {}
    }
}

#[tracing::instrument(level = "debug", skip_all)]
async fn handle_remove_from_playlist(
    weak: Weak<MainWindow>,
    state_manager: StateManager,
    songs: &[Song],
) {
    let song_ids: Vec<String> = songs
        .iter()
        .filter_map(|s| s.get_id().map(|id| id.to_string()))
        .collect();

    let mut selected_pid = String::new();
    if let Some(window) = weak.upgrade() {
        selected_pid = window
            .global::<PlaylistsPageProps>()
            .get_selected_playlist()
            .id
            .to_string();
    }

    if selected_pid.is_empty() {
        return;
    }

    tracing::debug!(
        "Removing {} songs from playlist '{}'",
        song_ids.len(),
        selected_pid
    );
    let db = state_manager.get_database().await;
    match db.remove_from_playlist(&selected_pid, &song_ids) {
        Ok(()) => {
            tracing::debug!(
                "Successfully removed songs from playlist '{}'",
                selected_pid
            );
        }
        Err(e) => {
            tracing::error!(
                "Failed to remove songs from playlist '{}': {:?}",
                selected_pid,
                e
            );
        }
    }

    let options = GetSongOptions {
        playlist: Some(Playlist {
            playlist_id: Some(selected_pid),
            ..Default::default()
        }),
        ..Default::default()
    };
    let updated_songs = match db.get_songs_by_options(options) {
        Ok(songs) => songs,
        Err(e) => {
            tracing::error!("Failed to fetch updated songs for playlist: {:?}", e);
            return;
        }
    };

    let _ = weak.upgrade_in_event_loop(move |window| {
        let songs_view: Vec<SongModel> = updated_songs.into_iter().map(Into::into).collect();
        let model = make_lazy_song_model(&window, &state_manager, songs_view);
        window.global::<PlaylistContentPageProps>().set_songs(model);
    });
}

#[tracing::instrument(level = "debug", skip_all)]
async fn handle_add_to_playlist(state_manager: &StateManager, songs: &[Song], playlist_id: &str) {
    tracing::debug!("Adding {} songs to playlist '{}'", songs.len(), playlist_id);
    let db = state_manager.get_database().await;
    match db.add_to_playlist(playlist_id, songs) {
        Ok(()) => {
            tracing::debug!("Successfully added songs to playlist '{}'", playlist_id);
        }
        Err(e) => {
            tracing::error!("Failed to add songs to playlist '{}': {:?}", playlist_id, e);
        }
    }
}

#[tracing::instrument(level = "debug", skip_all)]
async fn handle_goto_entity_by_id(
    weak: Weak<MainWindow>,
    state_manager: &StateManager,
    target_page: Pages,
    id: &str,
) {
    tracing::debug!("Navigating to {:?} with ID: '{}'", target_page, id);
    let db = state_manager.get_database().await;
    match target_page {
        Pages::AlbumContent => {
            let album = match db.get_entity_by_options(GetEntityOptions {
                album: Some(Album {
                    album_id: Some(id.to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }) {
                Ok(res) => res
                    .result
                    .and_then(|r| match r {
                        entity_result::Result::Albums(list) => list.albums.into_iter().next(),
                        _ => None,
                    })
                    .unwrap_or_else(|| Album {
                        album_id: Some(id.to_string()),
                        ..Default::default()
                    }),
                Err(e) => {
                    tracing::error!("Failed to fetch album by ID '{}': {:?}", id, e);
                    Album {
                        album_id: Some(id.to_string()),
                        ..Default::default()
                    }
                }
            };

            let _ = weak.upgrade_in_event_loop(move |window| {
                let album_model = AlbumModel::from(album);
                goto_album(&window, album_model);
            });
        }
        Pages::ArtistContent => {
            let artist = match db.get_entity_by_options(GetEntityOptions {
                artist: Some(Artist {
                    artist_id: Some(id.to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }) {
                Ok(res) => res
                    .result
                    .and_then(|r| match r {
                        entity_result::Result::Artists(list) => list.artists.into_iter().next(),
                        _ => None,
                    })
                    .unwrap_or_else(|| Artist {
                        artist_id: Some(id.to_string()),
                        ..Default::default()
                    }),
                Err(e) => {
                    tracing::error!("Failed to fetch artist by ID '{}': {:?}", id, e);
                    Artist {
                        artist_id: Some(id.to_string()),
                        ..Default::default()
                    }
                }
            };

            let _ = weak.upgrade_in_event_loop(move |window| {
                let artist_model = ArtistModel::from(artist);
                goto_artist(&window, artist_model);
            });
        }
        other => {
            tracing::error!("Unsupported goto entity target: {:?}", other);
        }
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn dispatch_song_context_action(
    main_window_weak: &Weak<MainWindow>,
    state_manager: &StateManager,
    song_models: &ModelRc<SongModel>,
    action_id: &str,
) {
    let state_manager = state_manager.clone();
    let songs: Vec<Song> = song_models.into_vec().into_iter().map(Song::from).collect();
    let action = action_id.to_string();
    let weak = main_window_weak.clone();

    tracing::debug!(
        "Dispatching song context menu action '{}' for {} songs",
        action,
        songs.len()
    );

    tokio::spawn(
        async move {
            match action.as_str() {
                "play_now" | "play_next" | "clear_queue_and_play" | "add_to_queue" => {
                    handle_playback_action(&state_manager, songs, &action).await;
                    return;
                }
                "remove_from_playlist" => {
                    handle_remove_from_playlist(weak, state_manager, &songs).await;
                    return;
                }
                _ => {}
            }

            if let Some(playlist_id) = action.strip_prefix("add_to_playlist:") {
                handle_add_to_playlist(&state_manager, &songs, playlist_id).await;
                return;
            }

            if let Some(album_id) = action.strip_prefix("goto_album:") {
                handle_goto_entity_by_id(weak, &state_manager, Pages::AlbumContent, album_id).await;
                return;
            }

            if let Some(artist_id) = action.strip_prefix("goto_artist:") {
                handle_goto_entity_by_id(weak, &state_manager, Pages::ArtistContent, artist_id)
                    .await;
            }
        }
        .instrument(tracing::debug_span!(
            "slint_cb_dispatch_song_context_action"
        )),
    );
}
