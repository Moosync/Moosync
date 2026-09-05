use slint::{ComponentHandle, ModelRc};
use songs_proto::moosync::types::{Genre, GetSongOptions, Song};
use state_manager::StateManager;

use crate::{
    GenreContentPageProps, GenresPageProps, MainWindow, SongModel,
    error::UiError,
    pages::PageHandler,
    utils::{EntityContentCoordinator, EntitySongProvider},
};

#[derive(Clone)]
pub struct GenreSongProvider;

#[async_trait::async_trait]
impl EntitySongProvider for GenreSongProvider {
    type Entity = Genre;

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_entity(main_window: &MainWindow) -> (Genre, String) {
        let genre: Genre = main_window
            .global::<GenresPageProps>()
            .get_selected_genre()
            .into();
        (genre, String::new())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_songs(main_window: &MainWindow, model: ModelRc<SongModel>) {
        main_window
            .global::<GenreContentPageProps>()
            .set_songs(model);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_local_songs(
        state_manager: &StateManager,
        genre: Genre,
    ) -> Result<Vec<Song>, UiError> {
        tracing::debug!("Fetching local songs for genre {:?}", genre.genre_name);
        let database = state_manager.get_database().await;
        let options = GetSongOptions {
            genre: Some(genre),
            ..Default::default()
        };
        database.get_songs_by_options(options).map_err(Into::into)
    }
}

pub struct GenreContentPageHandler<'a> {
    _main_window: &'a MainWindow,
    coordinator: EntityContentCoordinator<GenreSongProvider>,
}

impl<'a> GenreContentPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &'a MainWindow, state_manager: &'a StateManager) -> Self {
        Self {
            _main_window: main_window,
            coordinator: EntityContentCoordinator::new(main_window, state_manager),
        }
    }
}

impl<'a> PageHandler for GenreContentPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) { self.coordinator.on_show(); }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) { self.coordinator.on_hide(); }
}
