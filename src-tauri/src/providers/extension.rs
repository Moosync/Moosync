// Moosync
// Copyright (C) 2024, 2025  Moosync <support@moosync.app>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use std::fmt::Debug;

use async_trait::async_trait;
use futures::{channel::mpsc::UnboundedSender, SinkExt};
use serde_json::Value;
use tauri::AppHandle;
use tokio::sync::Mutex;
use types::{
    entities::{QueryableAlbum, QueryableArtist, QueryablePlaylist, SearchResult},
    errors::Result,
    providers::generic::{GenericProvider, Pagination, ProviderStatus},
    songs::Song,
    ui::extensions::{
        AccountLoginArgs, ContextMenuReturnType, CustomRequestReturnType, ExtensionDetail,
        ExtensionExtraEvent, ExtensionExtraEventArgs, ExtensionProviderScope, PackageNameArgs,
        PlaybackDetailsReturnType, PlaylistAndSongsReturnType, PlaylistReturnType,
        RecommendationsReturnType, SearchReturnType, SongReturnType, SongsWithPageTokenReturnType,
    },
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
        tracing::info!("parsing res {:?} as {}", res, stringify!($return_type));
        let res = serde_json::from_value::<$return_type>(res)?;
        tracing::info!("parsed res");
        res
    }};
}

pub struct ExtensionProvider {
    extension: ExtensionDetail,
    provides: Vec<ExtensionProviderScope>,
    app_handle: AppHandle,
    status_tx: Mutex<UnboundedSender<ProviderStatus>>,
}

impl ExtensionProvider {
    #[tracing::instrument(level = "trace", skip(extension, provides, app_handle, status_tx))]
    pub fn new(
        extension: ExtensionDetail,
        provides: Vec<ExtensionProviderScope>,
        app_handle: AppHandle,
        status_tx: UnboundedSender<ProviderStatus>,
    ) -> Self {
        Self {
            extension,
            provides,
            app_handle,
            status_tx: Mutex::new(status_tx),
        }
    }

    async fn get_accounts(&self) -> Result<()> {
        let extension_handler = get_extension_handler(&self.app_handle);
        let accounts = extension_handler
            .get_accounts(PackageNameArgs {
                package_name: self.extension.package_name.clone(),
            })
            .await?;

        for account in accounts {
            let _ = self
                .status_tx
                .lock()
                .await
                .send(ProviderStatus {
                    key: self.key(),
                    name: account.name,
                    user_name: account.username,
                    logged_in: account.logged_in,
                    bg_color: account.bg_color,
                    account_id: account.id,
                    scopes: self.get_provider_scopes().await.unwrap(),
                })
                .await;
        }

        Ok(())
    }
}

impl Debug for ExtensionProvider {
    #[tracing::instrument(level = "trace", skip(self, f))]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("ExtensionProvider")
            .field("extension", &self.extension)
            .field("provides", &self.provides)
            .finish()
    }
}

#[async_trait]
impl GenericProvider for ExtensionProvider {
    #[tracing::instrument(level = "trace", skip(self))]
    async fn initialize(&self) -> Result<()> {
        tracing::debug!(
            "Got extension provider scopes: {:?}",
            self.get_provider_scopes().await.unwrap()
        );
        if self.provides.contains(&ExtensionProviderScope::Accounts) {
            self.get_accounts().await
        } else {
            let _ = self
                .status_tx
                .lock()
                .await
                .send(ProviderStatus {
                    key: self.key(),
                    scopes: self.get_provider_scopes().await.unwrap(),
                    ..Default::default()
                })
                .await;
            Ok(())
        }
    }

    async fn get_provider_scopes(&self) -> Result<Vec<ExtensionProviderScope>> {
        Ok(self.provides.clone())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn key(&self) -> String {
        format!("extension:{}", self.extension.package_name)
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn match_id(&self, id: String) -> bool {
        id.starts_with(&format!("{}:", self.extension.package_name))
    }

    async fn requested_account_status(&self) -> Result<()> {
        self.get_accounts().await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn login(&self, account_id: String) -> Result<String> {
        let extension_handler = get_extension_handler(&self.app_handle);
        extension_handler
            .account_login(AccountLoginArgs {
                package_name: self.extension.package_name.clone(),
                account_id,
                login_status: true,
            })
            .await?;

        Ok(String::new())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn signout(&self, account_id: String) -> Result<()> {
        let extension_handler = get_extension_handler(&self.app_handle);
        extension_handler
            .account_login(AccountLoginArgs {
                package_name: self.extension.package_name.clone(),
                account_id,
                login_status: false,
            })
            .await?;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn authorize(&self, code: String) -> Result<()> {
        let _ = send_extension_event!(self, ExtensionExtraEvent::OauthCallback([code]), Value);
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn fetch_user_playlists(
        &self,
        pagination: Pagination,
    ) -> Result<(Vec<QueryablePlaylist>, Pagination)> {
        if !self.provides.contains(&ExtensionProviderScope::Playlists) {
            return Err("Extension does not have this capability".into());
        }
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
    #[tracing::instrument(level = "trace", skip(self))]
    async fn get_playlist_content(
        &self,
        playlist: QueryablePlaylist,
        pagination: Pagination,
    ) -> Result<(Vec<Song>, Pagination)> {
        if !self
            .provides
            .contains(&ExtensionProviderScope::PlaylistSongs)
        {
            return Err("Extension does not have this capability".into());
        }

        if playlist.playlist_id.is_none() {
            return Err("Playlist ID cannot be None".into());
        }

        if pagination.offset > 0 {
            return Ok((vec![], pagination.next_page()));
        }

        let res = send_extension_event!(
            self,
            ExtensionExtraEvent::RequestedPlaylistSongs(
                playlist.playlist_id.unwrap(),
                false,
                pagination.token.clone()
            ),
            SongsWithPageTokenReturnType
        );

        Ok((
            res.songs,
            pagination.next_page_wtoken(
                res.next_page_token
                    .map(|v| serde_json::to_string(&v).unwrap_or_default()),
            ),
        ))
    }
    #[tracing::instrument(level = "trace", skip(self))]
    async fn get_playback_url(&self, song: Song, _player: String) -> Result<String> {
        if !self
            .provides
            .contains(&ExtensionProviderScope::PlaybackDetails)
        {
            return Err("Extension does not have this capability".into());
        }

        if let Some(playback_url) = song.song.playback_url.clone() {
            if playback_url.starts_with("extension://") {
                let res = send_extension_event!(
                    self,
                    ExtensionExtraEvent::CustomRequest([playback_url.clone()]),
                    CustomRequestReturnType
                );
                tracing::info!("Got custom request {:?}", res);
                return Ok(res.redirect_url.unwrap_or(playback_url));
            }
        }

        let res = send_extension_event!(
            self,
            ExtensionExtraEvent::PlaybackDetailsRequested([song]),
            PlaybackDetailsReturnType
        );

        Ok(res.url)
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn search(&self, term: String) -> Result<SearchResult> {
        if !self.provides.contains(&ExtensionProviderScope::Search) {
            return Err("Extension does not have this capability".into());
        }

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

    #[tracing::instrument(level = "trace", skip(self))]
    async fn match_url(&self, url: String) -> Result<bool> {
        let res = send_extension_event!(
            self,
            ExtensionExtraEvent::RequestedPlaylistFromURL(url, false),
            PlaylistAndSongsReturnType
        );

        Ok(res.playlist.is_some())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn playlist_from_url(&self, url: String) -> Result<QueryablePlaylist> {
        if !self
            .provides
            .contains(&ExtensionProviderScope::PlaylistFromUrl)
        {
            return Err("Extension does not have this capability".into());
        }

        let res = send_extension_event!(
            self,
            ExtensionExtraEvent::RequestedPlaylistFromURL(url, false),
            PlaylistAndSongsReturnType
        );

        if let Some(playlist) = res.playlist {
            return Ok(playlist);
        }
        Err("Playlist not found".into())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn song_from_url(&self, url: String) -> Result<Song> {
        if !self.provides.contains(&ExtensionProviderScope::SongFromUrl) {
            return Err("Extension does not have this capability".into());
        }

        let res = send_extension_event!(
            self,
            ExtensionExtraEvent::RequestedSongFromURL(url, false),
            SongReturnType
        );

        if let Some(song) = res.song {
            return Ok(song);
        }

        Err("Song not found".into())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn get_suggestions(&self) -> Result<Vec<Song>> {
        if !self
            .provides
            .contains(&ExtensionProviderScope::Recommendations)
        {
            return Err("Extension does not have this capability".into());
        }

        let res = send_extension_event!(
            self,
            ExtensionExtraEvent::RequestedRecommendations,
            RecommendationsReturnType
        );

        Ok(res.songs)
    }

    async fn get_album_content(
        &self,
        album: QueryableAlbum,
        pagination: Pagination,
    ) -> Result<(Vec<Song>, Pagination)> {
        if !self.provides.contains(&ExtensionProviderScope::AlbumSongs) {
            return Err("Extension does not have this capability".into());
        }

        let res = send_extension_event!(
            self,
            ExtensionExtraEvent::RequestedAlbumSongs(album, pagination.token.clone()),
            SongsWithPageTokenReturnType
        );

        let pagination = pagination.next_page_wtoken(
            res.next_page_token
                .map(|t| serde_json::to_string(&t).unwrap_or_default()),
        );
        Ok((res.songs, pagination))
    }

    async fn get_artist_content(
        &self,
        artist: QueryableArtist,
        pagination: Pagination,
    ) -> Result<(Vec<Song>, Pagination)> {
        if !self.provides.contains(&ExtensionProviderScope::ArtistSongs) {
            return Err("Extension does not have this capability".into());
        }

        let res = send_extension_event!(
            self,
            ExtensionExtraEvent::RequestedArtistSongs(artist, pagination.token.clone()),
            SongsWithPageTokenReturnType
        );

        let pagination = pagination.next_page_wtoken(
            res.next_page_token
                .map(|t| serde_json::to_string(&t).unwrap_or_default()),
        );
        Ok((res.songs, pagination))
    }

    async fn get_lyrics(&self, song: Song) -> Result<String> {
        if !self.provides.contains(&ExtensionProviderScope::Lyrics) {
            return Err("Extension does not have this capability".into());
        }
        let res = send_extension_event!(self, ExtensionExtraEvent::RequestedLyrics([song]), String);

        Ok(res)
    }

    async fn get_song_context_menu(&self, song: Vec<Song>) -> Result<Vec<ContextMenuReturnType>> {
        if !self
            .provides
            .contains(&ExtensionProviderScope::SongContextMenu)
        {
            return Err("Extension does not have this capability".into());
        }
        let res = send_extension_event!(
            self,
            ExtensionExtraEvent::RequestedSongContextMenu([song]),
            Vec<ContextMenuReturnType>
        );

        Ok(res)
    }

    async fn get_playlist_context_menu(
        &self,
        playlist: QueryablePlaylist,
    ) -> Result<Vec<ContextMenuReturnType>> {
        if !self
            .provides
            .contains(&ExtensionProviderScope::PlaylistContextMenu)
        {
            return Err("Extension does not have this capability".into());
        }
        let res = send_extension_event!(
            self,
            ExtensionExtraEvent::RequestedPlaylistContextMenu([playlist]),
            Vec<ContextMenuReturnType>
        );

        Ok(res)
    }

    async fn trigger_context_menu_action(&self, action_id: String) -> Result<()> {
        if !self
            .provides
            .contains(&ExtensionProviderScope::PlaylistContextMenu)
            && !self
                .provides
                .contains(&ExtensionProviderScope::SongContextMenu)
        {
            return Err("Extension does not have this capability".into());
        }
        let res = send_extension_event!(
            self,
            ExtensionExtraEvent::ContextMenuAction([action_id]),
            ()
        );

        Ok(res)
    }

    async fn song_from_id(&self, id: String) -> Result<Song> {
        let res = send_extension_event!(self, ExtensionExtraEvent::RequestedSongFromId([id]), Song);

        Ok(res)
    }
}
