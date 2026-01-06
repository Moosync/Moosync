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

use extensions_proto::moosync::types::{
    ContextMenuActionRequest, ContextMenuReturnType, CustomRequest, ExtensionCommand,
    ExtensionDetail, ExtensionProviderScope, GetAccountsRequest, GetProviderScopesRequest,
    OauthCallbackRequest, PerformAccountLoginRequest, PlaybackDetailsRequestedRequest,
    ProviderStatus, RequestedAlbumSongsRequest, RequestedArtistSongsRequest,
    RequestedLyricsRequest, RequestedPlaylistContextMenuRequest, RequestedPlaylistFromUrlRequest,
    RequestedPlaylistSongsRequest, RequestedPlaylistsRequest, RequestedRecommendationsRequest,
    RequestedSearchResultRequest, RequestedSongContextMenuRequest, RequestedSongFromIdRequest,
    RequestedSongFromUrlRequest, extension_command, extension_command_response,
};
use songs_proto::moosync::types::{Album, Artist, Playlist, SearchResult, Song};
use tauri::AppHandle;
use types::{
    errors::{MoosyncError, Result},
    prelude::SongsExt,
    providers::generic::Pagination,
};

use crate::extensions::get_extension_handler;

macro_rules! send_extension_event {
    // Usage: send_extension_event!(self, request_struct, VariantName)
    ($self:ident, $req_data:expr, $variant:ident) => {{
        let extension_handler = get_extension_handler(&$self.app_handle);
        let event_wrapper = extension_command::Event::$variant($req_data);

        let res_wrapper = extension_handler
            .send_extension_command(ExtensionCommand {
                event: Some(event_wrapper),
                package_name: $self.extension.package_name.clone(),
            })
            .await
            .map_err(|e| MoosyncError::String(e.to_string()))?;

        // FIX: Use match instead of 'if let'.
        // We assume 'res_wrapper' is Option<ExtensionCommandResponse> based on your snippet.
        // If it is just ExtensionCommandResponse, remove the outer match.
        match res_wrapper {
            Some(res) => match res.response {
                Some(extension_command_response::Response::$variant(inner)) => {
                    tracing::debug!("Received response for {}", stringify!($variant));
                    inner
                }
                Some(unexpected) => {
                    let msg = format!(
                        "Expected response variant {}, got {:?}",
                        stringify!($variant),
                        unexpected
                    );
                    tracing::error!("{}", msg);
                    return Err(MoosyncError::String(msg));
                }
                None => {
                    return Err(MoosyncError::String(
                        "Received empty extension response (inner None)".into(),
                    ));
                }
            },
            None => {
                // This replaces your implicit else.
                // We must return an Error here, not Default::default(),
                // because the caller expects a concrete response type.
                return Err(MoosyncError::String(
                    "Received empty extension response (outer None)".into(),
                ));
            }
        }
    }};
}

pub struct ExtensionProvider {
    extension: ExtensionDetail,
    provides: Option<Vec<i32>>,
    app_handle: AppHandle,
}

impl ExtensionProvider {
    #[tracing::instrument(level = "debug", skip(extension, app_handle))]
    pub fn new(extension: ExtensionDetail, app_handle: AppHandle) -> Self {
        Self {
            extension,
            provides: None,
            app_handle,
        }
    }

    fn check_scope(&self, scope: ExtensionProviderScope) -> Result<()> {
        let has_scope = self
            .provides
            .as_ref()
            .map(|scopes| scopes.contains(&scope.into()))
            .unwrap_or(false);

        if has_scope {
            Ok(())
        } else {
            Err(MoosyncError::String(
                "Extension does not have this capability".into(),
            ))
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn key(&self) -> String {
        format!("extension:{}", self.extension.package_name)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn match_id(&self, id: String) -> bool {
        id.starts_with(&format!("{}:", self.extension.package_name))
    }

    pub async fn get_provider_scopes(&mut self) -> Result<Vec<i32>> {
        if let Some(provides) = &self.provides {
            Ok(provides.clone())
        } else {
            let res = send_extension_event!(self, GetProviderScopesRequest {}, GetProviderScopes);
            self.provides.replace(res.scopes.clone());
            Ok(res.scopes)
        }
    }

    pub async fn get_accounts(&mut self) -> Result<Vec<ProviderStatus>> {
        if self.check_scope(ExtensionProviderScope::Accounts).is_ok() {
            let res = send_extension_event!(self, GetAccountsRequest {}, GetAccounts);
            let scopes = self.get_provider_scopes().await.unwrap_or_default();
            Ok(res
                .accounts
                .into_iter()
                .map(|account| ProviderStatus {
                    key: self.key(),
                    name: account.name,
                    user_name: account.username,
                    logged_in: account.logged_in,
                    bg_color: account.bg_color,
                    account_id: account.id,
                    scopes: scopes.clone(),
                })
                .collect())
        } else {
            Ok(vec![ProviderStatus {
                key: self.key(),
                scopes: self.get_provider_scopes().await.unwrap_or_default(),
                name: self.extension.name.clone(),
                ..Default::default()
            }])
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn login(&self, account_id: String) -> Result<String> {
        let res = send_extension_event!(
            self,
            PerformAccountLoginRequest {
                account_id,
                login_status: true,
            },
            PerformAccountLogin
        );

        tracing::debug!("Got extension login response {:?}", res);
        Ok(res.status)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn signout(&self, account_id: String) -> Result<()> {
        let _ = send_extension_event!(
            self,
            PerformAccountLoginRequest {
                account_id,
                login_status: false,
            },
            PerformAccountLogin
        );
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn authorize(&self, code: String) -> Result<()> {
        let _ = send_extension_event!(
            self,
            OauthCallbackRequest { callback_uri: code },
            OauthCallback
        );
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn fetch_user_playlists(
        &self,
        pagination: Pagination,
    ) -> Result<(Vec<Playlist>, Pagination)> {
        self.check_scope(ExtensionProviderScope::Playlists)?;

        if pagination.offset > 0 {
            return Ok((vec![], pagination.next_page()));
        }

        let res = send_extension_event!(
            self,
            RequestedPlaylistsRequest { refresh: false },
            RequestedPlaylists
        );
        Ok((res.playlists, pagination.next_page()))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn get_playlist_content(
        &self,
        playlist: Playlist,
        pagination: Pagination,
    ) -> Result<(Vec<Song>, Pagination)> {
        self.check_scope(ExtensionProviderScope::PlaylistSongs)?;

        let playlist_id = playlist
            .playlist_id
            .ok_or(MoosyncError::String("Playlist ID cannot be None".into()))?;

        let res = send_extension_event!(
            self,
            RequestedPlaylistSongsRequest {
                id: playlist_id,
                refresh: false,
                page_token: pagination.token.clone(),
            },
            RequestedPlaylistSongs
        );

        let next_token = res.next_page_token.map(|v| v.to_string());

        Ok((res.songs, pagination.next_page_wtoken(next_token)))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn get_playback_url(&self, song: Song, _player: String) -> Result<String> {
        self.check_scope(ExtensionProviderScope::PlaybackDetails)?;

        if let Some(playback_url) = &song.get_playback_url()
            && playback_url.starts_with("extension://")
        {
            let res = send_extension_event!(
                self,
                CustomRequest {
                    request_id: playback_url.clone(),
                    payload: None
                },
                CustomRequest
            );
            tracing::info!("Got custom request {:?}", res);
            return Ok(res.redirect_url.unwrap_or(playback_url.clone()));
        }

        let res = send_extension_event!(
            self,
            PlaybackDetailsRequestedRequest { song: Some(song) },
            PlaybackDetailsRequested
        );

        Ok(res.url)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn search(&self, term: String) -> Result<SearchResult> {
        self.check_scope(ExtensionProviderScope::Search)?;

        let res = send_extension_event!(
            self,
            RequestedSearchResultRequest { query: term },
            RequestedSearchResult
        );

        Ok(SearchResult {
            songs: res.songs,
            artists: res.artists,
            playlists: res.playlists,
            albums: res.albums,
            genres: vec![],
        })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn match_url(&self, url: String) -> Result<bool> {
        let res = send_extension_event!(
            self,
            RequestedPlaylistFromUrlRequest {
                url,
                refresh: false
            },
            RequestedPlaylistFromUrl
        );

        Ok(res.playlist.is_some())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn playlist_from_url(&self, url: String) -> Result<Playlist> {
        self.check_scope(ExtensionProviderScope::PlaylistFromUrl)?;

        let res = send_extension_event!(
            self,
            RequestedPlaylistFromUrlRequest {
                url,
                refresh: false
            },
            RequestedPlaylistFromUrl
        );

        res.playlist
            .ok_or_else(|| MoosyncError::String("Playlist not found".into()))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn song_from_url(&self, url: String) -> Result<Song> {
        self.check_scope(ExtensionProviderScope::SongFromUrl)?;

        let res = send_extension_event!(
            self,
            RequestedSongFromUrlRequest {
                url,
                refresh: false
            },
            RequestedSongFromUrl
        );

        res.song
            .ok_or_else(|| MoosyncError::String("Song not found".into()))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn get_suggestions(&self) -> Result<Vec<Song>> {
        self.check_scope(ExtensionProviderScope::Recommendations)?;

        let res = send_extension_event!(
            self,
            RequestedRecommendationsRequest { refresh: false },
            RequestedRecommendations
        );

        Ok(res.songs)
    }

    pub async fn get_album_content(
        &self,
        album: Album,
        pagination: Pagination,
    ) -> Result<(Vec<Song>, Pagination)> {
        self.check_scope(ExtensionProviderScope::AlbumSongs)?;

        let res = send_extension_event!(
            self,
            RequestedAlbumSongsRequest {
                album: Some(album),
                page_token: pagination.token.clone(),
            },
            RequestedAlbumSongs
        );

        let next_token = res.next_page_token.map(|v| v.to_string());
        Ok((res.songs, pagination.next_page_wtoken(next_token)))
    }

    pub async fn get_artist_content(
        &self,
        artist: Artist,
        pagination: Pagination,
    ) -> Result<(Vec<Song>, Pagination)> {
        self.check_scope(ExtensionProviderScope::ArtistSongs)?;

        let res = send_extension_event!(
            self,
            RequestedArtistSongsRequest {
                artist: Some(artist),
                page_token: pagination.token.clone(),
            },
            RequestedArtistSongs
        );

        Ok((res.songs, pagination.next_page_wtoken(res.next_page_token)))
    }

    pub async fn get_lyrics(&self, song: Song) -> Result<String> {
        self.check_scope(ExtensionProviderScope::Lyrics)?;

        let res = send_extension_event!(
            self,
            RequestedLyricsRequest { song: Some(song) },
            RequestedLyrics
        );

        Ok(res.lyrics)
    }

    pub async fn get_song_context_menu(
        &self,
        songs: Vec<Song>,
    ) -> Result<Vec<ContextMenuReturnType>> {
        self.check_scope(ExtensionProviderScope::SongContextMenu)?;

        let res = send_extension_event!(
            self,
            RequestedSongContextMenuRequest { songs },
            RequestedSongContextMenu
        );

        Ok(vec![res.menu.unwrap_or_default()])
    }

    pub async fn get_playlist_context_menu(
        &self,
        playlist: Playlist,
    ) -> Result<Vec<ContextMenuReturnType>> {
        self.check_scope(ExtensionProviderScope::PlaylistContextMenu)?;

        let res = send_extension_event!(
            self,
            RequestedPlaylistContextMenuRequest {
                playlist: Some(playlist)
            },
            RequestedPlaylistContextMenu
        );

        Ok(vec![res.menu.unwrap_or_default()])
    }

    pub async fn trigger_context_menu_action(&self, action_id: String) -> Result<()> {
        self.check_scope(ExtensionProviderScope::PlaylistContextMenu)?;
        self.check_scope(ExtensionProviderScope::SongContextMenu)?;

        let _ = send_extension_event!(
            self,
            ContextMenuActionRequest { action_id },
            ContextMenuAction
        );

        Ok(())
    }

    pub async fn song_from_id(&self, id: String) -> Result<Song> {
        let res =
            send_extension_event!(self, RequestedSongFromIdRequest { id }, RequestedSongFromId);

        res.song
            .ok_or_else(|| MoosyncError::String("Song not found".into()))
    }

    pub async fn handle_extra_event(&self, event: extension_command::Event) -> Result<()> {
        let required_scope = match &event {
            extension_command::Event::SongAdded(_) | extension_command::Event::SongRemoved(_) => {
                ExtensionProviderScope::DatabaseSongEvents
            }
            extension_command::Event::PlaylistAdded(_)
            | extension_command::Event::PlaylistRemoved(_) => {
                ExtensionProviderScope::DatabasePlaylistEvents
            }
            extension_command::Event::VolumeChanged(_)
            | extension_command::Event::Seeked(_)
            | extension_command::Event::PlayerStateChanged(_) => {
                ExtensionProviderScope::PlayerUiEvents
            }
            extension_command::Event::SongQueueChanged(_)
            | extension_command::Event::SongChanged(_) => ExtensionProviderScope::PlayerDataEvents,
            extension_command::Event::Scrobble(_) => ExtensionProviderScope::Scrobble,

            _ => {
                return Err(MoosyncError::String(
                    "Event not mapped to a scope or manual send prohibited".into(),
                ));
            }
        };

        self.check_scope(required_scope)?;

        match event {
            extension_command::Event::SongAdded(req) => {
                let _ = send_extension_event!(self, req, SongAdded);
            }
            extension_command::Event::SongRemoved(req) => {
                let _ = send_extension_event!(self, req, SongRemoved);
            }
            extension_command::Event::PlaylistAdded(req) => {
                let _ = send_extension_event!(self, req, PlaylistAdded);
            }
            extension_command::Event::PlaylistRemoved(req) => {
                let _ = send_extension_event!(self, req, PlaylistRemoved);
            }
            extension_command::Event::VolumeChanged(req) => {
                let _ = send_extension_event!(self, req, VolumeChanged);
            }
            extension_command::Event::Seeked(req) => {
                let _ = send_extension_event!(self, req, Seeked);
            }
            extension_command::Event::PlayerStateChanged(req) => {
                let _ = send_extension_event!(self, req, PlayerStateChanged);
            }
            extension_command::Event::SongQueueChanged(req) => {
                let _ = send_extension_event!(self, req, SongQueueChanged);
            }
            extension_command::Event::SongChanged(req) => {
                let _ = send_extension_event!(self, req, SongChanged);
            }
            extension_command::Event::Scrobble(req) => {
                let _ = send_extension_event!(self, req, Scrobble);
            }
            _ => {}
        }

        Ok(())
    }
}

impl Debug for ExtensionProvider {
    #[tracing::instrument(level = "debug", skip(self, f))]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("ExtensionProvider")
            .field("extension", &self.extension)
            .field("provides", &self.provides)
            .finish()
    }
}
