use slint::{ComponentHandle, ModelRc, VecModel};
use songs_proto::moosync::types::{GetSongOptions, Playlist, Song};
use state_manager::StateManager;
use tracing::debug;
use types::prelude::SongsExt;

use crate::{
    ContextMenuCallbacks, ContextMenuItem, ContextMenuItems, MainWindow, PlaylistContentPageProps,
    PlaylistsPageProps, SongModel,
    error::UiError,
    pages::PageHandler,
    utils::{IntoVec, LazySongVecModel},
};

pub struct PlaylistContentPageHandler<'a> {
    main_window: &'a MainWindow,
    state_manager: &'a StateManager,
}

impl<'a> PlaylistContentPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &'a MainWindow, state_manager: &'a StateManager) -> Self {
        Self {
            main_window,
            state_manager,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_local_songs(
        state_manager: &StateManager,
        playlist: Playlist,
    ) -> Result<Vec<Song>, UiError> {
        let playlist_id = playlist.playlist_id.clone().unwrap_or_default();
        debug!("Fetching local songs for playlist ID: {}", playlist_id);
        let database = state_manager.get_database().await;
        let options = GetSongOptions {
            playlist: Some(playlist),
            ..Default::default()
        };
        database.get_songs_by_options(options).map_err(|e| e.into())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_extension_songs(
        state_manager: &StateManager,
        playlist_id: String,
        extension: String,
    ) -> Result<Vec<Song>, UiError> {
        debug!(
            "Fetching extension songs for playlist ID: {} from {}",
            playlist_id, extension
        );
        let handler = state_manager.get_extension_handler().await;
        let ext = handler.get_extension(&extension)?;
        let resp = ext
            .get_playlist_songs(
                extensions_proto::moosync::types::RequestedPlaylistSongsRequest {
                    id: playlist_id,
                    refresh: false,
                    page_token: None,
                },
            )
            .await?;
        Ok(resp.songs)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_songs(
        state_manager: &StateManager,
        playlist: Playlist,
        extension: String,
    ) -> Result<Vec<Song>, UiError> {
        if !extension.is_empty() {
            let playlist_id = playlist.playlist_id.clone().unwrap_or_default();
            return Self::fetch_extension_songs(state_manager, playlist_id, extension).await;
        }
        Self::fetch_local_songs(state_manager, playlist).await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_songs(
        main_window: &MainWindow,
        state_manager: &StateManager,
        songs: Vec<Song>,
        detail: Option<&extensions_proto::moosync::types::ExtensionDetail>,
    ) {
        debug!("Fetched {} songs for playlist", songs.len());
        let songs_view = songs
            .iter()
            .map(|s| crate::utils::to_song_model(s, detail))
            .collect::<Vec<_>>();
        let theme = main_window.global::<crate::Theme>();
        let cache_dir = state_manager.get_cache_dir();
        main_window
            .global::<PlaylistContentPageProps>()
            .set_songs(ModelRc::new(LazySongVecModel::new(
                songs_view,
                theme.get_songListItemHeight() as usize,
                theme.get_songListItemWidth() as usize,
                cache_dir,
            )));
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn register_context_menu_callbacks(&self) {
        let main_window_weak = self.main_window.as_weak();

        self.main_window
            .global::<ContextMenuCallbacks>()
            .on_get_playlist_song_menu_items(move |_song_models| {
                let Some(main_window) = main_window_weak.upgrade() else {
                    return ModelRc::default();
                };

                let all_items: Vec<ContextMenuItem> = main_window
                    .global::<ContextMenuItems>()
                    .invoke_get_playlist_song_items()
                    .into_vec();

                ModelRc::new(VecModel::from(all_items))
            });

        let state_manager = self.state_manager.clone();
        self.main_window
            .global::<ContextMenuCallbacks>()
            .on_playlist_song_action(move |song_models, playlist_id, action_id| {
                Self::dispatch_action(
                    &state_manager,
                    &song_models,
                    playlist_id.as_str(),
                    action_id.as_str(),
                );
            });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn dispatch_action(
        state_manager: &StateManager,
        song_models: &ModelRc<SongModel>,
        playlist_id: &str,
        action_id: &str,
    ) {
        let state_manager = state_manager.clone();
        let songs: Vec<Song> = song_models.into_vec().into_iter().map(Song::from).collect();
        let song_ids: Vec<String> = songs
            .iter()
            .filter_map(|s| s.get_id().map(|id| id.to_string()))
            .collect();
        let pid = playlist_id.to_string();
        let action = action_id.to_string();

        tokio::spawn(async move {
            match action.as_str() {
                "play_now" => {
                    let mut player = state_manager.get_player_handler_mut().await;
                    player.play_now(songs);
                }
                "add_to_queue" | "play_next" => {
                    let mut player = state_manager.get_player_handler_mut().await;
                    player.add_to_queue(songs);
                }
                "remove_from_playlist" => {
                    let db = state_manager.get_database().await;
                    let _ = db.remove_from_playlist(&pid, &song_ids);
                }
                _ => {}
            }
        });
    }
}

impl<'a> PageHandler for PlaylistContentPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) { self.register_context_menu_callbacks(); }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) {
        let selected = self
            .main_window
            .global::<PlaylistsPageProps>()
            .get_selected_playlist();
        let extension = selected.extension.to_string();
        let playlist: Playlist = selected.into();

        tokio::spawn({
            let state_manager = self.state_manager.clone();
            let main_window_weak = self.main_window.as_weak();
            async move {
                let detail = if !extension.is_empty() {
                    let handler_ext = state_manager.get_extension_handler().await;
                    handler_ext
                        .get_extension(&extension)
                        .ok()
                        .map(|e| e.get_extension_detail())
                } else {
                    None
                };
                match Self::fetch_songs(&state_manager, playlist, extension).await {
                    Ok(songs) => {
                        let _ = main_window_weak.upgrade_in_event_loop(move |main_window| {
                            Self::set_songs(&main_window, &state_manager, songs, detail.as_ref());
                        });
                    }
                    Err(e) => {
                        tracing::error!("Failed to fetch playlist songs: {:?}", e)
                    }
                }
            }
        });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) {
        self.main_window
            .global::<PlaylistContentPageProps>()
            .set_songs(ModelRc::default());
    }
}
