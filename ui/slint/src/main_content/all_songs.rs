use slint::{ComponentHandle, ModelRc};
use songs_proto::moosync::types::{GetSongOptions, SearchableSong, Song};
use state_manager::StateManager;

use crate::{
    AllSongsPageProps, ContextMenuCallbacks, MainWindow, Theme,
    error::UiError,
    pages::PageHandler,
    utils::{LazySongVecModel, build_song_context_menu_items, dispatch_song_context_action},
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
        let state_manager_clone = self.state_manager.clone();

        self.main_window
            .global::<ContextMenuCallbacks>()
            .on_get_song_menu_items(move |song_models| {
                let Some(main_window) = main_window_weak.upgrade() else {
                    return ModelRc::default();
                };

                build_song_context_menu_items(&main_window, &state_manager_clone, &song_models)
            });

        let state_manager = self.state_manager.clone();
        let main_window_weak = self.main_window.as_weak();
        self.main_window
            .global::<ContextMenuCallbacks>()
            .on_song_action(move |song_models, action_id| {
                dispatch_song_context_action(
                    &main_window_weak,
                    &state_manager,
                    &song_models,
                    action_id.as_str(),
                );
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
        let songs_view = songs.into_iter().map(Into::into).collect::<Vec<_>>();

        let theme = main_window.global::<Theme>();
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
