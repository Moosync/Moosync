use std::fmt::Debug;

use async_trait::async_trait;
use tauri::{AppHandle, Manager};
use types::{
    entities::{QueryablePlaylist, SearchResult},
    errors::errors::Result,
    extensions::{
        ExtensionDetail, ExtensionExtraEvent, ExtensionExtraEventArgs, ExtensionProviderScope,
        PlaybackDetailsReturnType, PlaylistReturnType, SearchReturnType,
        SongsWithPageTokenReturnType,
    },
    providers::generic::{GenericProvider, Pagination, ProviderStatus},
    songs::Song,
};

use crate::extensions::get_extension_handler;

macro_rules! send_extension_event {
    ($self:ident, $data:expr, $return_type:ty) => {{
        let extension_handler = get_extension_handler(&$self.app_handle);
        let res = extension_handler
            .send_extra_event(ExtensionExtraEventArgs {
                data: $data,
                package_name: $self.extension.package_name.clone(),
            })
            .await?;
        let res = serde_json::from_value::<$return_type>(res)?;
        res
    }};
}

#[derive(Clone)]
pub struct ExtensionProvider {
    extension: ExtensionDetail,
    provides: Vec<ExtensionProviderScope>,
    app_handle: AppHandle,
}

impl ExtensionProvider {
    pub fn new(
        extension: ExtensionDetail,
        provides: Vec<ExtensionProviderScope>,
        app_handle: AppHandle,
    ) -> Self {
        Self {
            extension,
            provides,
            app_handle,
        }
    }
}

impl Debug for ExtensionProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("ExtensionProvider")
            .field("extension", &self.extension)
            .field("provides", &self.provides)
            .finish()
    }
}

#[async_trait]
impl GenericProvider for ExtensionProvider {
    async fn initialize(&mut self) -> Result<()> {
        Ok(())
    }
    fn key(&self) -> String {
        format!("extension:{}", self.extension.package_name)
    }
    fn match_id(&self, id: String) -> bool {
        id.starts_with(&format!("{}:", self.extension.package_name))
    }

    async fn login(&mut self) -> Result<()> {
        Ok(())
    }
    async fn authorize(&mut self, code: String) -> Result<()> {
        Ok(())
    }

    async fn fetch_user_details(&self) -> Result<ProviderStatus> {
        Ok(ProviderStatus::default())
    }
    async fn fetch_user_playlists(
        &self,
        pagination: Pagination,
    ) -> Result<(Vec<QueryablePlaylist>, Pagination)> {
        if pagination.offset > 0 {
            return Ok((vec![], pagination.next_page()));
        }

        let res = send_extension_event!(
            self,
            ExtensionExtraEvent::RequestedPlaylists([false]),
            PlaylistReturnType
        );
        Ok((res.playlists, pagination.next_page()))
    }
    async fn get_playlist_content(
        &self,
        playlist_id: String,
        pagination: Pagination,
    ) -> Result<(Vec<Song>, Pagination)> {
        if pagination.offset > 0 {
            return Ok((vec![], pagination.next_page()));
        }

        let res = send_extension_event!(
            self,
            ExtensionExtraEvent::RequestedPlaylistSongs(
                playlist_id,
                false,
                pagination.token.clone()
            ),
            SongsWithPageTokenReturnType
        );

        Ok((
            res.songs,
            pagination.next_page_wtoken(
                res.next_page_token
                    .map(|v| serde_json::from_value(v).unwrap()),
            ),
        ))
    }
    async fn get_playback_url(&self, song: Song, player: String) -> Result<String> {
        let res = send_extension_event!(
            self,
            ExtensionExtraEvent::PlaybackDetailsRequested([song]),
            PlaybackDetailsReturnType
        );

        Ok(res.url)
    }

    async fn search(&self, term: String) -> Result<SearchResult> {
        let res = send_extension_event!(
            self,
            ExtensionExtraEvent::RequestedSearchResult([term]),
            SearchReturnType
        );

        Ok(SearchResult {
            songs: res.songs,
            artists: res.artists,
            playlists: res.playlists,
            albums: res.albums,
            genres: vec![],
        })
    }
}
