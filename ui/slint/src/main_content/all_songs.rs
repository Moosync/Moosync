use slint::{ComponentHandle, Model, ModelRc, VecModel};
use songs_proto::moosync::types::{GetSongOptions, SearchableSong, Song};
use state_manager::StateManager;
use tracing::debug;

use crate::{
    AllSongsPageProps, ContextMenuCallbacks, ContextMenuItem, ContextMenuItems, MainWindow,
    SongModel,
    error::UiError,
    pages::PageHandler,
    utils::{IntoVec, LazySongVecModel},
};

pub struct AllSongsPageHandler<'a> {
    main_window: &'a MainWindow,
    state_manager: &'a StateManager,
}

impl<'a> AllSongsPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &'a MainWindow, state_manager: &'a StateManager) -> Self {
        Self {
            main_window,
            state_manager,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn register_context_menu_callbacks(&self) {
        let main_window_weak = self.main_window.as_weak();

        self.main_window
            .global::<ContextMenuCallbacks>()
            .on_get_all_songs_menu_items(move |song_models| {
                let Some(main_window) = main_window_weak.upgrade() else {
                    return ModelRc::default();
                };

                let mut all_items: Vec<ContextMenuItem> = main_window
                    .global::<ContextMenuItems>()
                    .invoke_get_all_songs_items()
                    .into_vec();

                let first_song = song_models.row_data(0);
                if let Some(song) = first_song {
                    if !song.path.is_empty() {
                        all_items.push(ContextMenuItem {
                            action_id: "open_in_file_manager".into(),
                            title: "Show in File Manager".into(),
                            icon: crate::utils::default_folder_icon(),
                        });
                    }
                }

                ModelRc::new(VecModel::from(all_items))
            });

        let state_manager = self.state_manager.clone();
        self.main_window
            .global::<ContextMenuCallbacks>()
            .on_all_songs_action(move |song_models, action_id| {
                Self::dispatch_action(&state_manager, &song_models, action_id.as_str());
            });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn dispatch_action(
        state_manager: &StateManager,
        song_models: &ModelRc<SongModel>,
        action_id: &str,
    ) {
        let state_manager = state_manager.clone();
        let songs: Vec<Song> = song_models.into_vec().into_iter().map(Song::from).collect();
        let action = action_id.to_string();

        tokio::spawn(async move {
            let mut player = state_manager.get_player_handler_mut().await;
            match action.as_str() {
                "play_now" => {
                    player.play_now(songs);
                }
                "add_to_queue" | "play_next" => {
                    player.add_to_queue(songs);
                }
                _ => {}
            }
        });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn get_songs_from_db(state_manager: &StateManager) -> Result<Vec<Song>, UiError> {
        let database = state_manager.get_database().await;
        let songs = database.get_songs_by_options(GetSongOptions {
            song: Some(SearchableSong::default()),
            ..Default::default()
        })?;
        Ok(songs)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_songs(state_manager: &StateManager) -> Result<Vec<Song>, UiError> {
        Self::get_songs_from_db(state_manager).await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_songs(main_window: &MainWindow, state_manager: &StateManager, songs: Vec<Song>) {
        debug!("Setting songs");
        let songs_view = songs
            .iter()
            .map(|s| crate::utils::to_song_model(s, None))
            .collect::<Vec<_>>();

        let theme = main_window.global::<crate::Theme>();
        let cache_dir = state_manager.get_cache_dir();
        main_window
            .global::<AllSongsPageProps>()
            .set_songs(ModelRc::new(LazySongVecModel::new(
                songs_view,
                theme.get_songListItemHeight() as usize,
                theme.get_songListItemWidth() as usize,
                cache_dir,
            )));
    }
}

impl<'a> PageHandler for AllSongsPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) { self.register_context_menu_callbacks(); }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) {
        tokio::spawn({
            let state_manager = self.state_manager.clone();
            let main_window_weak = self.main_window.as_weak();
            async move {
                if let Ok(songs) = Self::fetch_songs(&state_manager).await {
                    let _ = main_window_weak.upgrade_in_event_loop(move |main_window| {
                        Self::set_songs(&main_window, &state_manager, songs);
                    });
                }
            }
        });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) {
        self.main_window
            .global::<AllSongsPageProps>()
            .set_songs(ModelRc::default());
    }
}
