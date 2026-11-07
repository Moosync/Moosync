use std::{path::Path, rc::Rc, sync::Arc};

use database::Database;
use extensions::{Extension, ExtensionError, ExtensionGroupExt};
use extensions_proto::moosync::types::{
    ContextMenuActionRequest, ContextMenuReturnType, ExtensionProviderScope,
    RequestedAlbumSongsRequest, RequestedArtistSongsRequest, RequestedPlaylistContextMenuRequest,
    RequestedSongContextMenuRequest,
};
use futures::StreamExt;
use slint::{ComponentHandle, Image, Model, ModelRc, SharedString, VecModel, Weak};
use songs_proto::moosync::types::{
    Album, Artist, Genre, GetEntityOptions, GetSongOptions, Playlist, Song, entity_result,
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
    AlbumModel, AppProps, ArtistModel, ContextMenuItem, ContextMenuItems, ContextSubMenuItem,
    GenreModel, MainWindow, Pages, PlaylistContentPageProps, PlaylistModel, PlaylistsPageProps,
    SavePlaylistState, SongModel,
};

pub const ACTION_ADD_TO_PLAYLIST_NEW: &str = "add_to_playlist:__new__";
pub const PREFIX_ADD_TO_PLAYLIST: &str = "add_to_playlist:";
pub const ACTION_ADD_TO_PLAYLIST: &str = "add_to_playlist";
pub const ACTION_REMOVE_FROM_PLAYLIST: &str = "remove_from_playlist";
pub const PREFIX_GOTO_ALBUM: &str = "goto_album:";
pub const PREFIX_GOTO_ARTIST: &str = "goto_artist:";
pub const ACTION_GOTO_ARTIST: &str = "goto_artist";
pub const PREFIX_GOTO: &str = "goto_";
pub const PREFIX_EXTENSION: &str = "extension:";
pub const ACTION_PLAY_NOW: &str = "play_now";
pub const ACTION_PLAY_NEXT: &str = "play_next";
pub const ACTION_CLEAR_QUEUE_AND_PLAY: &str = "clear_queue_and_play";
pub const ACTION_ADD_TO_QUEUE: &str = "add_to_queue";

#[derive(Clone)]
enum ExtensionMenuRequest {
    Songs(Vec<Song>),
    Playlist(Playlist),
}

/// Creates a top-level context menu item without a submenu.
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

/// Creates a nested context submenu item.
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

/// Creates a top-level context menu item with a populated submenu.
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

/// Appends "Go to album" and "Go to artist" items based on the first selected
/// song.
#[tracing::instrument(level = "debug", skip_all)]
fn populate_navigation_items(
    items: &mut Vec<ContextMenuItem>,
    main_window: &MainWindow,
    song_models: &ModelRc<SongModel>,
) {
    let Some(first_song) = song_models.row_data(0) else {
        return;
    };
    let menu_items = main_window.global::<ContextMenuItems>();

    if !first_song.album_name.is_empty() {
        let title = menu_items.invoke_get_goto_album_title(first_song.album_name.clone());
        let icon = menu_items.invoke_get_album_icon();
        items.push(make_context_menu_item(
            format!("{PREFIX_GOTO_ALBUM}{}", first_song.album_id),
            title,
            icon,
        ));
    }

    let artist_icon = menu_items.invoke_get_artist_icon();
    let artist_sub_items: Vec<ContextSubMenuItem> = (0..first_song.artists.row_count())
        .filter_map(|i| first_song.artists.row_data(i))
        .map(|a| {
            make_context_sub_item(
                format!("{PREFIX_GOTO_ARTIST}{}", a.id),
                a.title,
                artist_icon.clone(),
            )
        })
        .collect();

    if !artist_sub_items.is_empty() {
        let title = menu_items.invoke_get_goto_artists_title();
        items.push(make_context_submenu_item(
            ACTION_GOTO_ARTIST,
            title,
            artist_icon,
            artist_sub_items,
        ));
    }
}

/// Fetches all playlists from the database.
#[tracing::instrument(level = "debug", skip_all)]
fn fetch_playlists(db: &Database) -> Vec<Playlist> {
    let res = match db.get_entity_by_options(GetEntityOptions {
        playlist: Some(Playlist::default()),
        ..Default::default()
    }) {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to fetch playlists for context menu: {:?}", e);
            return Vec::new();
        }
    };
    match res.result {
        Some(entity_result::Result::Playlists(list)) => list.playlists,
        _ => Vec::new(),
    }
}

/// Asynchronously populates and inserts the "Add to playlist" submenu.
#[tracing::instrument(level = "debug", skip_all)]
fn attach_playlist_submenu(
    vec_model: Rc<VecModel<ContextMenuItem>>,
    main_window_weak: Weak<MainWindow>,
    state_manager: &'static StateManager,
) {
    let _ = slint::spawn_local(
        async move {
            let playlists = fetch_playlists(&state_manager.get_database());
            let Some(window) = main_window_weak.upgrade() else {
                return;
            };

            let menu_items = window.global::<ContextMenuItems>();
            let playlist_icon = menu_items.invoke_get_playlist_icon();
            let plus_icon = menu_items.invoke_get_plus_icon();

            let mut sub_items = Vec::with_capacity(playlists.len() + 1);
            sub_items.push(make_context_sub_item(
                ACTION_ADD_TO_PLAYLIST_NEW,
                menu_items.invoke_get_new_playlist_title(),
                plus_icon,
            ));

            for p in playlists {
                let pid = p.playlist_id.unwrap_or_default();
                sub_items.push(make_context_sub_item(
                    format!("{PREFIX_ADD_TO_PLAYLIST}{pid}"),
                    p.playlist_name,
                    playlist_icon.clone(),
                ));
            }

            let title = menu_items.invoke_get_add_to_playlist_title();
            let item =
                make_context_submenu_item(ACTION_ADD_TO_PLAYLIST, title, playlist_icon, sub_items);
            let insert_idx = (0..vec_model.row_count())
                .position(|i| {
                    vec_model
                        .row_data(i)
                        .is_some_and(|existing| existing.action_id.starts_with(PREFIX_GOTO))
                })
                .unwrap_or_else(|| vec_model.row_count());
            vec_model.insert(insert_idx, item);
        }
        .in_current_span(),
    );
}

/// Requests a context menu item from a single extension for the given target.
#[tracing::instrument(level = "debug", skip_all)]
async fn fetch_extension_menu(
    ext: Arc<Extension>,
    request: ExtensionMenuRequest,
) -> (
    Arc<Extension>,
    Result<Option<ContextMenuReturnType>, ExtensionError>,
) {
    let res = match request {
        ExtensionMenuRequest::Songs(songs) => ext
            .get_song_context_menu(RequestedSongContextMenuRequest { songs })
            .await
            .map(|r| r.menu),
        ExtensionMenuRequest::Playlist(playlist) => ext
            .get_playlist_context_menu(RequestedPlaylistContextMenuRequest {
                playlist: Some(playlist),
            })
            .await
            .map(|r| r.menu),
    };
    (ext, res)
}

/// Asynchronously fetches context menu items from matching extensions and
/// appends them to the UI model.
#[tracing::instrument(level = "debug", skip_all)]
fn attach_extension_menu_items(
    vec_model: Rc<VecModel<ContextMenuItem>>,
    default_ext_icon: Image,
    state_manager: &'static StateManager,
    request: ExtensionMenuRequest,
) {
    let scope = match request {
        ExtensionMenuRequest::Songs(_) => ExtensionProviderScope::SongContextMenu,
        ExtensionMenuRequest::Playlist(_) => ExtensionProviderScope::PlaylistContextMenu,
    };

    let _ = slint::spawn_local(
        async move {
            let Ok(extensions) = tokio::spawn(
                async move {
                    state_manager
                        .get_extension_handler()
                        .get_extensions_with_scope(scope)
                        .await
                }
                .in_current_span(),
            )
            .await
            else {
                return;
            };

            let mut tasks = extensions.with(|ext| {
                let req = request.clone();
                tokio::spawn(fetch_extension_menu(ext, req).in_current_span())
            });

            while let Some(Ok((ext, res))) = tasks.next().await {
                match res {
                    Ok(Some(menu)) if !menu.action_id.is_empty() && !menu.name.is_empty() => {
                        let icon = if !menu.icon.is_empty() {
                            Image::load_from_path(Path::new(&menu.icon))
                                .unwrap_or_else(|_| default_ext_icon.clone())
                        } else {
                            default_ext_icon.clone()
                        };
                        vec_model.push(make_context_menu_item(menu.action_id, menu.name, icon));
                    }
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!(
                            "Failed to get context menu from extension '{}': {:?}",
                            ext.get_package_name(),
                            e
                        );
                    }
                }
            }
        }
        .in_current_span(),
    );
}

/// Attaches navigation items, the playlist submenu, and extension items to a
/// song context menu.
#[tracing::instrument(level = "debug", skip_all)]
fn finalize_song_context_menu(
    main_window: &MainWindow,
    state_manager: &'static StateManager,
    song_models: &ModelRc<SongModel>,
    mut items: Vec<ContextMenuItem>,
) -> ModelRc<ContextMenuItem> {
    populate_navigation_items(&mut items, main_window, song_models);

    let vec_model = Rc::new(VecModel::from(items));
    let model_rc = ModelRc::new(vec_model.clone());

    attach_playlist_submenu(vec_model.clone(), main_window.as_weak(), state_manager);

    let songs: Vec<Song> = song_models.into_vec().into_iter().map(Song::from).collect();
    if !songs.is_empty() {
        let ext_icon = main_window
            .global::<ContextMenuItems>()
            .invoke_get_extension_icon();
        attach_extension_menu_items(
            vec_model,
            ext_icon,
            state_manager,
            ExtensionMenuRequest::Songs(songs),
        );
    }

    model_rc
}

/// Builds the context menu item model for a playlist.
#[tracing::instrument(level = "debug", skip_all)]
pub fn build_playlist_context_menu_items(
    main_window: &MainWindow,
    state_manager: &'static StateManager,
    playlist_models: &ModelRc<PlaylistModel>,
) -> ModelRc<ContextMenuItem> {
    let menu_items = main_window.global::<ContextMenuItems>();
    let all_items = menu_items.invoke_get_playlists_items().into_vec();

    let vec_model = Rc::new(VecModel::from(all_items));
    let model_rc = ModelRc::new(vec_model.clone());

    if let Some(first_playlist) = playlist_models.row_data(0) {
        let ext_icon = menu_items.invoke_get_extension_icon();
        attach_extension_menu_items(
            vec_model,
            ext_icon,
            state_manager,
            ExtensionMenuRequest::Playlist(Playlist::from(first_playlist)),
        );
    }

    model_rc
}

/// Builds the context menu item model for songs in library or content views.
#[tracing::instrument(level = "debug", skip_all)]
pub fn build_song_context_menu_items(
    main_window: &MainWindow,
    state_manager: &'static StateManager,
    song_models: &ModelRc<SongModel>,
) -> ModelRc<ContextMenuItem> {
    let menu_items = main_window.global::<ContextMenuItems>();
    let mut all_items = menu_items.invoke_get_all_songs_items().into_vec();

    if main_window.global::<AppProps>().get_active_page() == Pages::PlaylistContent {
        let title = menu_items.invoke_get_remove_from_playlist_title();
        let icon = menu_items.invoke_get_remove_icon();
        all_items.push(make_context_menu_item(
            ACTION_REMOVE_FROM_PLAYLIST,
            title,
            icon,
        ));
    }

    finalize_song_context_menu(main_window, state_manager, song_models, all_items)
}

/// Builds the context menu item model for songs in the playback queue.
#[tracing::instrument(level = "debug", skip_all)]
pub fn build_queue_context_menu_items(
    main_window: &MainWindow,
    state_manager: &'static StateManager,
    song_models: &ModelRc<SongModel>,
    _idx: i32,
) -> ModelRc<ContextMenuItem> {
    let all_items = main_window
        .global::<ContextMenuItems>()
        .invoke_get_queue_items()
        .into_vec();

    finalize_song_context_menu(main_window, state_manager, song_models, all_items)
}

/// Executes a player queue action on the selected songs.
#[tracing::instrument(level = "debug", skip_all)]
async fn handle_playback_action(state_manager: &StateManager, songs: Vec<Song>, action: &str) {
    let player = state_manager.get_player_handler();
    match action {
        ACTION_PLAY_NOW => player.play_now(songs).await,
        ACTION_PLAY_NEXT => player.play_next(songs).await,
        ACTION_CLEAR_QUEUE_AND_PLAY => player.clear_and_play(songs).await,
        ACTION_ADD_TO_QUEUE => player.add_to_queue(songs).await,
        _ => {}
    }
}

/// Removes the selected songs from the active playlist and refreshes the
/// playlist view.
#[tracing::instrument(level = "debug", skip_all)]
async fn handle_remove_from_playlist(
    weak: Weak<MainWindow>,
    state_manager: &'static StateManager,
    songs: &[Song],
) {
    let Some(selected_pid) = weak
        .upgrade()
        .map(|w| {
            w.global::<PlaylistsPageProps>()
                .get_selected_playlist()
                .id
                .to_string()
        })
        .filter(|id| !id.is_empty())
    else {
        return;
    };

    let song_ids: Vec<String> = songs
        .iter()
        .filter_map(|s| s.get_id().map(|id| id.to_string()))
        .collect();

    let db = state_manager.get_database();
    if let Err(e) = db.remove_from_playlist(&selected_pid, &song_ids) {
        tracing::error!(
            "Failed to remove songs from playlist '{}': {:?}",
            selected_pid,
            e
        );
    }

    let updated_songs = match db.get_songs_by_options(GetSongOptions {
        playlist: Some(Playlist {
            playlist_id: Some(selected_pid),
            ..Default::default()
        }),
        ..Default::default()
    }) {
        Ok(songs) => songs,
        Err(e) => {
            tracing::error!("Failed to fetch updated songs for playlist: {:?}", e);
            return;
        }
    };

    let _ = weak.upgrade_in_event_loop(move |window| {
        let songs_view: Vec<SongModel> = updated_songs.into_iter().map(Into::into).collect();
        let model = make_lazy_song_model(&window, state_manager, songs_view);
        window.global::<PlaylistContentPageProps>().set_songs(model);
    });
}

/// Adds the selected songs to the specified playlist in the database.
#[tracing::instrument(level = "debug", skip_all)]
async fn handle_add_to_playlist(state_manager: &StateManager, songs: &[Song], playlist_id: &str) {
    if let Err(e) = state_manager
        .get_database()
        .add_to_playlist(playlist_id, songs)
    {
        tracing::error!("Failed to add songs to playlist '{}': {:?}", playlist_id, e);
    }
}

/// Looks up an album by ID from the database, falling back to an ID-only stub.
#[tracing::instrument(level = "debug", skip_all)]
fn fetch_album_by_id(db: &Database, id: &str) -> Album {
    let fallback = || Album {
        album_id: Some(id.to_string()),
        ..Default::default()
    };
    let res = match db.get_entity_by_options(GetEntityOptions {
        album: Some(fallback()),
        ..Default::default()
    }) {
        Ok(res) => res,
        Err(e) => {
            tracing::error!("Failed to fetch album by ID '{}': {:?}", id, e);
            return fallback();
        }
    };
    let Some(entity_result::Result::Albums(list)) = res.result else {
        return fallback();
    };
    list.albums.into_iter().next().unwrap_or_else(fallback)
}

/// Looks up an artist by ID from the database, falling back to an ID-only stub.
#[tracing::instrument(level = "debug", skip_all)]
fn fetch_artist_by_id(db: &Database, id: &str) -> Artist {
    let fallback = || Artist {
        artist_id: Some(id.to_string()),
        ..Default::default()
    };
    let res = match db.get_entity_by_options(GetEntityOptions {
        artist: Some(fallback()),
        ..Default::default()
    }) {
        Ok(res) => res,
        Err(e) => {
            tracing::error!("Failed to fetch artist by ID '{}': {:?}", id, e);
            return fallback();
        }
    };
    let Some(entity_result::Result::Artists(list)) = res.result else {
        return fallback();
    };
    list.artists.into_iter().next().unwrap_or_else(fallback)
}

/// Navigates to the album or artist content page for the given entity ID.
#[tracing::instrument(level = "debug", skip_all)]
async fn handle_goto_entity_by_id(
    weak: Weak<MainWindow>,
    state_manager: &StateManager,
    target_page: Pages,
    id: &str,
) {
    let db = state_manager.get_database();
    match target_page {
        Pages::AlbumContent => {
            let album = fetch_album_by_id(&db, id);
            let _ = weak.upgrade_in_event_loop(move |window| {
                goto_album(&window, AlbumModel::from(album));
            });
        }
        Pages::ArtistContent => {
            let artist = fetch_artist_by_id(&db, id);
            let _ = weak.upgrade_in_event_loop(move |window| {
                goto_artist(&window, ArtistModel::from(artist));
            });
        }
        other => {
            tracing::error!("Unsupported goto entity target: {:?}", other);
        }
    }
}

/// Forwards an `extension:<pkg>:<id>` context menu action to its target
/// extension.
#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_extension_context_action(state_manager: &StateManager, action: &str) -> bool {
    let Some(rest) = action.strip_prefix(PREFIX_EXTENSION) else {
        return false;
    };
    let Some((pkg, _)) = rest.split_once(':') else {
        return false;
    };

    let ext = match state_manager.get_extension_handler().get_extension(pkg) {
        Ok(ext) => ext,
        Err(e) => {
            tracing::error!(
                "Extension '{}' not found for context menu action '{}': {:?}",
                pkg,
                action,
                e
            );
            return true;
        }
    };

    if let Err(e) = ext
        .context_menu_action(ContextMenuActionRequest {
            action_id: action.to_string(),
        })
        .await
    {
        tracing::error!(
            "Extension '{}' failed to handle context menu action '{}': {:?}",
            pkg,
            action,
            e
        );
    }
    true
}

/// Executes a song-related action on the resolved list of songs.
#[tracing::instrument(level = "debug", skip_all)]
pub async fn execute_song_context_action(
    weak: Weak<MainWindow>,
    state_manager: &'static StateManager,
    songs: Vec<Song>,
    action: &str,
) {
    if handle_extension_context_action(state_manager, action).await {
        return;
    }
    if action == ACTION_ADD_TO_PLAYLIST_NEW {
        let _ = weak.upgrade_in_event_loop(move |window| {
            let song_models: Vec<SongModel> = songs.into_iter().map(SongModel::from).collect();
            let state = window.global::<SavePlaylistState>();
            state.set_songs(ModelRc::new(VecModel::from(song_models)));
            state.set_show_modal(true);
        });
        return;
    }
    if let Some(playlist_id) = action.strip_prefix(PREFIX_ADD_TO_PLAYLIST) {
        handle_add_to_playlist(state_manager, &songs, playlist_id).await;
        return;
    }
    if let Some(album_id) = action.strip_prefix(PREFIX_GOTO_ALBUM) {
        handle_goto_entity_by_id(weak, state_manager, Pages::AlbumContent, album_id).await;
        return;
    }
    if let Some(artist_id) = action.strip_prefix(PREFIX_GOTO_ARTIST) {
        handle_goto_entity_by_id(weak, state_manager, Pages::ArtistContent, artist_id).await;
        return;
    }

    match action {
        ACTION_PLAY_NOW | ACTION_PLAY_NEXT | ACTION_CLEAR_QUEUE_AND_PLAY | ACTION_ADD_TO_QUEUE => {
            handle_playback_action(state_manager, songs, action).await;
        }
        ACTION_REMOVE_FROM_PLAYLIST => {
            handle_remove_from_playlist(weak, state_manager, &songs).await;
        }
        _ => {}
    }
}

/// Dispatches a selected song context menu action to the appropriate handler.
#[tracing::instrument(level = "debug", skip_all)]
pub fn dispatch_song_context_action(
    main_window_weak: &Weak<MainWindow>,
    state_manager: &'static StateManager,
    song_models: &ModelRc<SongModel>,
    action_id: &str,
) {
    let songs: Vec<Song> = song_models.into_vec().into_iter().map(Song::from).collect();
    let action = action_id.to_string();
    let weak = main_window_weak.clone();

    tokio::spawn(
        async move {
            execute_song_context_action(weak, state_manager, songs, &action).await;
        }
        .instrument(tracing::debug_span!(
            "slint_cb_dispatch_song_context_action"
        )),
    );
}

#[tracing::instrument(level = "debug", skip_all)]
async fn fetch_songs_for_album(state_manager: &StateManager, album: Album) -> Vec<Song> {
    if let Some(extension_id) = album.extension.as_ref() {
        if let Ok(extension) = state_manager
            .get_extension_handler()
            .get_extension(extension_id)
        {
            match extension
                .get_album_songs(RequestedAlbumSongsRequest {
                    album: Some(album.clone()),
                    page_token: None,
                })
                .await
            {
                Ok(response) => return response.songs,
                Err(error) => {
                    tracing::error!("Failed to fetch album songs from extension: {:?}", error)
                }
            }
        }
    }
    match state_manager
        .get_database()
        .get_songs_by_options(GetSongOptions {
            album: Some(album),
            ..Default::default()
        }) {
        Ok(songs) => songs,
        Err(error) => {
            tracing::error!("Failed to fetch songs for album: {:?}", error);
            Vec::new()
        }
    }
}

#[tracing::instrument(level = "debug", skip_all)]
async fn fetch_songs_for_artist(state_manager: &StateManager, artist: Artist) -> Vec<Song> {
    if let Some(extension_id) = artist.extension.as_ref() {
        if let Ok(extension) = state_manager
            .get_extension_handler()
            .get_extension(extension_id)
        {
            match extension
                .get_artist_songs(RequestedArtistSongsRequest {
                    artist: Some(artist.clone()),
                    page_token: None,
                })
                .await
            {
                Ok(response) => return response.songs,
                Err(error) => {
                    tracing::error!("Failed to fetch artist songs from extension: {:?}", error)
                }
            }
        }
    }
    match state_manager
        .get_database()
        .get_songs_by_options(GetSongOptions {
            artist: Some(artist),
            ..Default::default()
        }) {
        Ok(songs) => songs,
        Err(error) => {
            tracing::error!("Failed to fetch songs for artist: {:?}", error);
            Vec::new()
        }
    }
}

#[tracing::instrument(level = "debug", skip_all)]
async fn fetch_songs_for_genre(state_manager: &StateManager, genre: Genre) -> Vec<Song> {
    match state_manager
        .get_database()
        .get_songs_by_options(GetSongOptions {
            genre: Some(genre),
            ..Default::default()
        }) {
        Ok(songs) => songs,
        Err(error) => {
            tracing::error!("Failed to fetch songs for genre: {:?}", error);
            Vec::new()
        }
    }
}

/// Builds the context menu item model for an album.
#[tracing::instrument(level = "debug", skip_all)]
pub fn build_album_context_menu_items(
    main_window: &MainWindow,
    state_manager: &'static StateManager,
    _album_models: &ModelRc<AlbumModel>,
) -> ModelRc<ContextMenuItem> {
    let menu_items = main_window.global::<ContextMenuItems>();
    let items = menu_items.invoke_get_all_songs_items().into_vec();
    let vec_model = Rc::new(VecModel::from(items));
    let model_rc = ModelRc::new(vec_model.clone());
    attach_playlist_submenu(vec_model, main_window.as_weak(), state_manager);
    model_rc
}

/// Dispatches an album context menu action to the appropriate handler.
#[tracing::instrument(level = "debug", skip_all)]
pub fn dispatch_album_context_action(
    main_window_weak: &Weak<MainWindow>,
    state_manager: &'static StateManager,
    album_models: &ModelRc<AlbumModel>,
    action_id: &str,
) {
    let albums: Vec<Album> = album_models
        .into_vec()
        .into_iter()
        .map(Album::from)
        .collect();
    let action = action_id.to_string();
    let weak = main_window_weak.clone();

    tokio::spawn(
        async move {
            let mut all_songs = Vec::new();
            for album in albums {
                all_songs.extend(fetch_songs_for_album(state_manager, album).await);
            }
            execute_song_context_action(weak, state_manager, all_songs, &action).await;
        }
        .instrument(tracing::debug_span!(
            "slint_cb_dispatch_album_context_action"
        )),
    );
}

/// Builds the context menu item model for an artist.
#[tracing::instrument(level = "debug", skip_all)]
pub fn build_artist_context_menu_items(
    main_window: &MainWindow,
    state_manager: &'static StateManager,
    _artist_models: &ModelRc<ArtistModel>,
) -> ModelRc<ContextMenuItem> {
    let menu_items = main_window.global::<ContextMenuItems>();
    let items = menu_items.invoke_get_all_songs_items().into_vec();
    let vec_model = Rc::new(VecModel::from(items));
    let model_rc = ModelRc::new(vec_model.clone());
    attach_playlist_submenu(vec_model, main_window.as_weak(), state_manager);
    model_rc
}

/// Dispatches an artist context menu action to the appropriate handler.
#[tracing::instrument(level = "debug", skip_all)]
pub fn dispatch_artist_context_action(
    main_window_weak: &Weak<MainWindow>,
    state_manager: &'static StateManager,
    artist_models: &ModelRc<ArtistModel>,
    action_id: &str,
) {
    let artists: Vec<Artist> = artist_models
        .into_vec()
        .into_iter()
        .map(Artist::from)
        .collect();
    let action = action_id.to_string();
    let weak = main_window_weak.clone();

    tokio::spawn(
        async move {
            let mut all_songs = Vec::new();
            for artist in artists {
                all_songs.extend(fetch_songs_for_artist(state_manager, artist).await);
            }
            execute_song_context_action(weak, state_manager, all_songs, &action).await;
        }
        .instrument(tracing::debug_span!(
            "slint_cb_dispatch_artist_context_action"
        )),
    );
}

/// Builds the context menu item model for a genre.
#[tracing::instrument(level = "debug", skip_all)]
pub fn build_genre_context_menu_items(
    main_window: &MainWindow,
    state_manager: &'static StateManager,
    _genre_models: &ModelRc<GenreModel>,
) -> ModelRc<ContextMenuItem> {
    let menu_items = main_window.global::<ContextMenuItems>();
    let items = menu_items.invoke_get_all_songs_items().into_vec();
    let vec_model = Rc::new(VecModel::from(items));
    let model_rc = ModelRc::new(vec_model.clone());
    attach_playlist_submenu(vec_model, main_window.as_weak(), state_manager);
    model_rc
}

/// Dispatches a genre context menu action to the appropriate handler.
#[tracing::instrument(level = "debug", skip_all)]
pub fn dispatch_genre_context_action(
    main_window_weak: &Weak<MainWindow>,
    state_manager: &'static StateManager,
    genre_models: &ModelRc<GenreModel>,
    action_id: &str,
) {
    let genres: Vec<Genre> = genre_models
        .into_vec()
        .into_iter()
        .map(Genre::from)
        .collect();
    let action = action_id.to_string();
    let weak = main_window_weak.clone();

    tokio::spawn(
        async move {
            let mut all_songs = Vec::new();
            for genre in genres {
                all_songs.extend(fetch_songs_for_genre(state_manager, genre).await);
            }
            execute_song_context_action(weak, state_manager, all_songs, &action).await;
        }
        .instrument(tracing::debug_span!(
            "slint_cb_dispatch_genre_context_action"
        )),
    );
}
