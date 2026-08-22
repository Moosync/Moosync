use slint::{ComponentHandle, ModelRc};
use songs_proto::moosync::types::{Artist, GetSongOptions, Song};
use state_manager::StateManager;
use tracing::debug;

use crate::{
    ArtistContentPageProps, ArtistsPageProps, MainWindow, error::UiError, pages::PageHandler,
    utils::LazySongVecModel,
};

pub struct ArtistContentPageHandler<'a> {
    main_window: &'a MainWindow,
    state_manager: &'a StateManager,
}

impl<'a> ArtistContentPageHandler<'a> {
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
        artist: Artist,
    ) -> Result<Vec<Song>, UiError> {
        let artist_id = artist.artist_id.clone().unwrap_or_default();
        debug!("Fetching local songs for artist ID: {}", artist_id);
        let database = state_manager.get_database().await;
        let options = GetSongOptions {
            artist: Some(artist),
            ..Default::default()
        };
        database.get_songs_by_options(options).map_err(|e| e.into())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_extension_songs(
        state_manager: &StateManager,
        artist: Artist,
        extension: String,
    ) -> Result<Vec<Song>, UiError> {
        debug!(
            "Fetching extension songs for artist ID: {:?} from {}",
            artist.artist_id, extension
        );
        let handler = state_manager.get_extension_handler().await;
        let ext = handler.get_extension(&extension)?;
        let resp = ext
            .get_artist_songs(
                extensions_proto::moosync::types::RequestedArtistSongsRequest {
                    artist: Some(artist),
                    page_token: None,
                },
            )
            .await?;
        Ok(resp.songs)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_songs(
        state_manager: &StateManager,
        artist: Artist,
        extension: String,
    ) -> Result<Vec<Song>, UiError> {
        if !extension.is_empty() {
            return Self::fetch_extension_songs(state_manager, artist, extension).await;
        }
        Self::fetch_local_songs(state_manager, artist).await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_songs(
        main_window: &MainWindow,
        state_manager: &StateManager,
        songs: Vec<Song>,
        detail: Option<&extensions_proto::moosync::types::ExtensionDetail>,
    ) {
        debug!("Fetched {} songs for artist", songs.len());
        let songs_view = songs
            .iter()
            .map(|s| crate::utils::to_song_model(s, detail))
            .collect::<Vec<_>>();
        let theme = main_window.global::<crate::Theme>();
        let cache_dir = state_manager.get_cache_dir();
        main_window
            .global::<ArtistContentPageProps>()
            .set_songs(ModelRc::new(LazySongVecModel::new(
                songs_view,
                theme.get_songListItemHeight() as usize,
                theme.get_songListItemWidth() as usize,
                cache_dir,
            )));
    }
}

impl<'a> PageHandler for ArtistContentPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) {}

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) {
        let selected = self
            .main_window
            .global::<ArtistsPageProps>()
            .get_selected_artist();
        let artist = Artist {
            artist_id: Some(selected.id.to_string()),
            artist_name: Some(selected.title.to_string()),
            artist_coverpath: Some(selected.coverPathUrl.to_string()),
            artist_mbid: Some(selected.mbid.to_string()),
            sanitized_artist_name: Some(selected.sanitized_name.to_string()),
            artist_song_count: selected.songs_count as f64,
        };
        let extension = selected.extension.to_string();

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
                match Self::fetch_songs(&state_manager, artist, extension).await {
                    Ok(songs) => {
                        let _ = main_window_weak.upgrade_in_event_loop(move |main_window| {
                            Self::set_songs(&main_window, &state_manager, songs, detail.as_ref());
                        });
                    }
                    Err(e) => {
                        tracing::error!("Failed to fetch artist songs: {:?}", e)
                    }
                }
            }
        });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) {
        self.main_window
            .global::<ArtistContentPageProps>()
            .set_songs(ModelRc::default());
    }
}
