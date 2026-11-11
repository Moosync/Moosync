use std::time::Duration;

use slint::{ComponentHandle, ModelRc, Weak};
use songs_proto::moosync::types::{GetSongOptions, SearchableSong, Song};
use state_manager::StateManager;
use tracing::Instrument;
use types::debounce::DebounceHandle;

use crate::{
    AllSongsPageProps, AppProps, ContextMenuCallbacks, MainWindow, Pages,
    error::UiError,
    pages::PageHandler,
    utils::{build_song_context_menu_items, dispatch_song_context_action, make_lazy_song_model},
};

pub struct AllSongsPageHandler {
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
}

impl AllSongsPageHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &'static MainWindow, state_manager: &'static StateManager) -> Self {
        Self {
            main_window,
            state_manager,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn register_context_menu_callbacks(&self) {
        let main_window_weak = self.main_window.as_weak();
        let state_manager = self.state_manager;

        self.main_window
            .global::<ContextMenuCallbacks>()
            .on_get_song_menu_items(move |song_models| {
                let Some(main_window) = main_window_weak.upgrade() else {
                    return ModelRc::default();
                };

                build_song_context_menu_items(&main_window, state_manager, &song_models)
            });

        let main_window_weak = self.main_window.as_weak();
        self.main_window
            .global::<ContextMenuCallbacks>()
            .on_song_action(move |song_models, action_id| {
                dispatch_song_context_action(
                    &main_window_weak,
                    state_manager,
                    &song_models,
                    action_id.as_str(),
                );
            });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) async fn fetch_songs(state_manager: &StateManager) -> Result<Vec<Song>, UiError> {
        tracing::debug!("Fetching all songs from database");
        let database = state_manager.get_database();
        let songs = database.get_songs_by_options(GetSongOptions {
            song: Some(SearchableSong::default()),
            ..Default::default()
        })?;
        Ok(songs)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn set_songs(
        main_window: &MainWindow,
        state_manager: &StateManager,
        songs: Vec<Song>,
    ) {
        let songs_view = songs.into_iter().map(Into::into).collect::<Vec<_>>();
        let model = make_lazy_song_model(main_window, state_manager, songs_view);
        main_window.global::<AllSongsPageProps>().set_songs(model);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn refresh_songs(
        main_window_weak: Weak<MainWindow>,
        state_manager: &'static StateManager,
    ) {
        let is_active = main_window_weak
            .upgrade()
            .map(|main_window| {
                main_window.global::<AppProps>().get_active_page() == Pages::AllSongs
            })
            .unwrap_or(false);
        if !is_active {
            return;
        }

        tokio::spawn(
            async move {
                match Self::fetch_songs(state_manager).await {
                    Ok(songs) => {
                        tracing::debug!("AllSongsPage: fetched {} songs", songs.len());
                        let _ = main_window_weak.upgrade_in_event_loop(move |main_window| {
                            if main_window.global::<AppProps>().get_active_page() == Pages::AllSongs
                            {
                                Self::set_songs(&main_window, state_manager, songs);
                            }
                        });
                    }
                    Err(e) => {
                        tracing::error!("AllSongsPage: Failed to fetch all songs: {:?}", e);
                    }
                }
            }
            .in_current_span(),
        );
    }
}

impl PageHandler for AllSongsPageHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) {
        self.register_context_menu_callbacks();

        let main_window_weak = self.main_window.as_weak();
        let state_manager = self.state_manager;
        let debounce = DebounceHandle::new();

        self.state_manager
            .get_database()
            .on_songs_added
            .listen(move |_| {
                let main_window_weak = main_window_weak.clone();
                debounce.debounce(
                    Duration::from_millis(150),
                    async move {
                        Self::refresh_songs(main_window_weak, state_manager);
                    }
                    .in_current_span(),
                );
            });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) {
        tracing::debug!("AllSongsPage: on_show triggered");
        Self::refresh_songs(self.main_window.as_weak(), self.state_manager);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) {
        tracing::debug!("AllSongsPage: on_hide clearing songs");
        self.main_window
            .global::<AllSongsPageProps>()
            .set_songs(ModelRc::default());
    }
}
