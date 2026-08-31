use std::rc::Rc;

use slint::{ComponentHandle, Image, Model, ModelRc, SharedString, VecModel, Weak};
use songs_proto::moosync::types::{
    Album, Artist, GetEntityOptions, GetSongOptions, Playlist, Song, entity_result,
};
use state_manager::StateManager;
use types::prelude::SongsExt;

use super::{default_empty_icon, lazy_model::LazySongVecModel, models::IntoVec};
use crate::{
    AlbumsPageProps, ArtistsPageProps, ContextMenuItem, ContextMenuItems, ContextSubMenuItem,
    MainWindow, Pages, PlaylistContentPageProps, PlaylistsPageProps, SongModel, Theme,
};

#[tracing::instrument(level = "debug", skip_all)]
pub fn make_context_menu_item(
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
pub fn make_context_sub_item(
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
pub fn make_context_submenu_item(
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
            default_empty_icon(),
        ));
    }

    let artist_sub_items: Vec<ContextSubMenuItem> = (0..first_song.artists.row_count())
        .filter_map(|i| first_song.artists.row_data(i))
        .map(|a| {
            let action_id = format!("goto_artist:{}", a.id);
            make_context_sub_item(action_id, a.title, default_empty_icon())
        })
        .collect();

    if !artist_sub_items.is_empty() {
        let title = main_window
            .global::<ContextMenuItems>()
            .invoke_get_goto_artists_title();
        items.push(make_context_submenu_item(
            "goto_artist",
            title,
            default_empty_icon(),
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
    let _ = slint::spawn_local(async move {
        let db = state_manager.get_database().await;
        let Ok(res) = db.get_entity_by_options(GetEntityOptions {
            playlist: Some(Playlist::default()),
            ..Default::default()
        }) else {
            return;
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
                make_context_sub_item(action_id, p.playlist_name, default_empty_icon())
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
            default_empty_icon(),
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
    });
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
            icon: default_empty_icon(),
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

    let db = state_manager.get_database().await;
    let _ = db.remove_from_playlist(&selected_pid, &song_ids);
    let options = GetSongOptions {
        playlist: Some(Playlist {
            playlist_id: Some(selected_pid),
            ..Default::default()
        }),
        ..Default::default()
    };
    let Ok(updated_songs) = db.get_songs_by_options(options) else {
        return;
    };

    let cache_dir = state_manager.get_cache_dir();
    let _ = weak.upgrade_in_event_loop(move |window| {
        let songs_view = updated_songs
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>();
        let theme = window.global::<Theme>();
        window
            .global::<PlaylistContentPageProps>()
            .set_songs(ModelRc::new(LazySongVecModel::new(
                songs_view,
                theme.get_songListItemHeight() as usize,
                theme.get_songListItemWidth() as usize,
                cache_dir,
            )));
    });
}

#[tracing::instrument(level = "debug", skip_all)]
async fn handle_add_to_playlist(state_manager: &StateManager, songs: &[Song], playlist_id: &str) {
    let db = state_manager.get_database().await;
    let _ = db.add_to_playlist(playlist_id, songs);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn handle_goto_album(weak: Weak<MainWindow>, state_manager: &StateManager, album_id: &str) {
    let db = state_manager.get_database().await;
    let Ok(res) = db.get_entity_by_options(GetEntityOptions {
        album: Some(Album {
            album_id: Some(album_id.to_string()),
            ..Default::default()
        }),
        ..Default::default()
    }) else {
        return;
    };

    let Some(entity_result::Result::Albums(list)) = res.result else {
        return;
    };
    let Some(album) = list.albums.into_iter().next() else {
        return;
    };

    let _ = weak.upgrade_in_event_loop(move |window| {
        window
            .global::<AlbumsPageProps>()
            .set_selected_album(album.into());
        window.set_active_page(Pages::AlbumContent);
    });
}

#[tracing::instrument(level = "debug", skip_all)]
async fn handle_goto_artist(weak: Weak<MainWindow>, state_manager: &StateManager, artist_id: &str) {
    let db = state_manager.get_database().await;
    let Ok(res) = db.get_entity_by_options(GetEntityOptions {
        artist: Some(Artist {
            artist_id: Some(artist_id.to_string()),
            ..Default::default()
        }),
        ..Default::default()
    }) else {
        return;
    };

    let Some(entity_result::Result::Artists(list)) = res.result else {
        return;
    };
    let Some(artist) = list.artists.into_iter().next() else {
        return;
    };

    let _ = weak.upgrade_in_event_loop(move |window| {
        window
            .global::<ArtistsPageProps>()
            .set_selected_artist(artist.into());
        window.set_active_page(Pages::ArtistContent);
    });
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

    tokio::spawn(async move {
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
            handle_goto_album(weak, &state_manager, album_id).await;
            return;
        }

        if let Some(artist_id) = action.strip_prefix("goto_artist:") {
            handle_goto_artist(weak, &state_manager, artist_id).await;
        }
    });
}
