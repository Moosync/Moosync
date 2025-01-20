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

use std::{
    collections::{HashMap, HashSet},
    io::{BufRead, BufReader},
    net::TcpListener,
    thread,
};

use async_trait::async_trait;

use chrono::{DateTime, TimeDelta};
use futures::{channel::mpsc::UnboundedSender, SinkExt, StreamExt};
use oauth2::{CsrfToken, PkceCodeVerifier};
use preferences::preferences::PreferenceConfig;
use regex::Regex;
use rspotify::{
    clients::{BaseClient, OAuthClient},
    model::{
        AlbumId, ArtistId, FullAlbum, FullArtist, FullTrack, Id, Image, PlaylistId,
        PlaylistTracksRef, SearchType, SimplifiedAlbum, SimplifiedArtist, SimplifiedPlaylist,
        SimplifiedTrack, SubscriptionLevel, TrackId,
    },
    AuthCodePkceSpotify, Token,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager, State};
use types::{
    entities::{EntityInfo, QueryableAlbum, QueryableArtist, QueryablePlaylist, SearchResult},
    errors::Result,
    oauth::OAuth2Client,
    providers::generic::{Pagination, ProviderStatus},
    songs::{QueryableSong, Song, SongType},
    ui::extensions::{ContextMenuReturnType, ExtensionProviderScope},
};
use types::{errors::MoosyncError, providers::generic::GenericProvider};
use url::Url;

use crate::{librespot::initialize_librespot, oauth::handler::OAuthHandler};

use super::common::{
    authorize, get_oauth_client, login, refresh_login, LoginArgs, OAuthClientArgs, TokenHolder,
};

macro_rules! search_and_parse_all {
    ($client:expr, $term:expr, [$(($type:expr, $variant:path, $parse_fn:expr, $result_vec:expr)),*]) => {{
        $(
            if let Ok($variant(items)) = $client.search($term, $type, None, None, Some(50), Some(0)).await {
                for item in items.items {
                    $parse_fn(item, &mut $result_vec);
                }
            }
        )*
    }};
}

#[derive(Debug, Clone, Default)]
struct SpotifyConfig {
    client_secret: Option<String>,
    client_id: Option<String>,
    redirect_uri: &'static str,
    scopes: Vec<&'static str>,
    tokens: Option<TokenHolder>,
}

#[derive(Debug)]
pub struct SpotifyProvider {
    app: AppHandle,
    config: SpotifyConfig,
    verifier: Option<(OAuth2Client, PkceCodeVerifier, CsrfToken)>,
    api_client: Option<AuthCodePkceSpotify>,
    status_tx: UnboundedSender<ProviderStatus>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ArtistExtraInfo {
    artist_id: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SpotifyExtraInfo {
    spotify: ArtistExtraInfo,
}

impl SpotifyProvider {
    #[tracing::instrument(level = "trace", skip(app, status_tx))]
    pub fn new(app: AppHandle, status_tx: UnboundedSender<ProviderStatus>) -> Self {
        Self {
            app,
            config: SpotifyConfig::default(),
            verifier: None,
            api_client: None,
            status_tx,
        }
    }
}

impl SpotifyProvider {
    async fn get_provider_status(&self, user_name: Option<String>) -> ProviderStatus {
        ProviderStatus {
            key: self.key(),
            name: "Spotify".into(),
            user_name: user_name.clone(),
            logged_in: user_name.is_some(),
            bg_color: "#07C330".into(),
            account_id: "spotify".into(),
            scopes: self.get_provider_scopes().await.unwrap(),
        }
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn get_oauth_client(&self) -> OAuth2Client {
        get_oauth_client(OAuthClientArgs {
            auth_url: "https://accounts.spotify.com/authorize".to_string(),
            token_url: "https://accounts.spotify.com/api/token".to_string(),
            redirect_url: self.config.redirect_uri.to_string(),
            client_id: self.config.client_id.clone().unwrap(),
            client_secret: self.config.client_secret.clone().unwrap_or_default(),
        })
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn create_api_client(&mut self) {
        tracing::debug!("Creating spotify api client");
        if let Some(token) = &self.config.tokens {
            self.api_client = Some(AuthCodePkceSpotify::from_token(Token {
                access_token: token.access_token.clone(),
                expires_in: TimeDelta::seconds(token.expires_in.try_into().unwrap()),
                expires_at: Some(DateTime::from_timestamp_millis(token.expires_at).unwrap()),
                refresh_token: Some(token.refresh_token.clone()),
                scopes: HashSet::from_iter(self.config.scopes.iter().map(|v| v.to_string())),
            }));

            let res = self.fetch_user_details().await;
            let mut is_spotify_premium = false;
            if let Ok((res, is_premium)) = res {
                let _ = self.status_tx.send(res).await;
                is_spotify_premium = is_premium;
            } else {
                let _ = self
                    .status_tx
                    .send(self.get_provider_status(Some("".into())).await)
                    .await;
            }

            if is_spotify_premium {
                tracing::debug!("Initializing librespot");
                if let Err(err) = initialize_librespot(self.app.clone(), token.access_token.clone())
                {
                    tracing::error!("Error initializing librespot {:?}", err);
                }
            }
        }
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn refresh_login(&mut self) -> Result<()> {
        tracing::debug!("Refreshing spotify login");
        self.config.tokens = Some(
            refresh_login(
                "MoosyncSpotifyRefreshToken",
                self.get_oauth_client(),
                &self.app,
            )
            .await?,
        );
        self.create_api_client().await;

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self, playlist))]
    fn parse_playlist(&self, playlist: SimplifiedPlaylist) -> QueryablePlaylist {
        QueryablePlaylist {
            playlist_id: Some(format!("spotify-playlist:{}", playlist.id.id())),
            playlist_name: playlist.name,
            playlist_coverpath: playlist.images.first().map(|i| i.url.clone()),
            playlist_song_count: playlist.tracks.total as f64,
            extension: Some(self.key()),
            ..Default::default()
        }
    }

    #[tracing::instrument(level = "trace", skip(self, artist))]
    fn parse_artists(
        &self,
        artist: SimplifiedArtist,
        images: Option<Vec<Image>>,
    ) -> QueryableArtist {
        QueryableArtist {
            artist_id: Some(format!("spotify-artist:{}", artist.id.clone().unwrap())),
            artist_name: Some(artist.name),
            artist_coverpath: images.and_then(|i| i.first().map(|im| im.url.clone())),
            artist_extra_info: Some(EntityInfo(
                serde_json::to_string(&SpotifyExtraInfo {
                    spotify: ArtistExtraInfo {
                        artist_id: artist.id.clone().unwrap().to_string(),
                    },
                })
                .unwrap(),
            )),
            ..Default::default()
        }
    }

    #[tracing::instrument(level = "trace", skip(self, album))]
    fn parse_album(&self, album: SimplifiedAlbum) -> QueryableAlbum {
        QueryableAlbum {
            album_id: Some(format!("spotify-album:{}", album.id.clone().unwrap())),
            album_name: Some(album.name),
            album_artist: album.artists.first().map(|a| a.name.clone()),
            album_coverpath_high: album.images.first().map(|i| i.url.clone()),
            album_coverpath_low: album.images.last().map(|i| i.url.clone()),
            ..Default::default()
        }
    }

    #[tracing::instrument(level = "trace", skip(self, item))]
    fn parse_playlist_item(&self, item: FullTrack) -> Song {
        let id = item.id.unwrap().to_string();
        Song {
            song: QueryableSong {
                _id: Some(format!("spotify:{}", id)),
                title: Some(item.name),
                duration: Some(item.duration.num_seconds() as f64),
                type_: SongType::SPOTIFY,
                url: Some(id.clone()),
                song_cover_path_high: item.album.images.first().map(|i| i.url.clone()),
                playback_url: Some(id),
                track_no: Some(item.disc_number as f64),
                provider_extension: Some(self.key()),
                ..Default::default()
            },
            album: if item.album.id.is_some() {
                Some(self.parse_album(item.album))
            } else {
                None
            },
            artists: Some(
                item.artists
                    .into_iter()
                    .map(|a| self.parse_artists(a, None))
                    .collect(),
            ),
            ..Default::default()
        }
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn fetch_user_details(&self) -> Result<(ProviderStatus, bool)> {
        tracing::info!("Fetching user details for spotify");
        if let Some(api_client) = &self.api_client {
            let token = api_client.token.lock().await.unwrap();
            drop(token);

            let user = api_client.current_user().await?;
            let mut is_premium = false;
            if let Some(subscription) = user.product {
                if subscription == SubscriptionLevel::Premium {
                    is_premium = true
                }
            }
            return Ok((
                self.get_provider_status(user.display_name).await,
                is_premium,
            ));
        }

        Err("API client not initialized".into())
    }

    fn get_full_track(&self, track: SimplifiedTrack) -> FullTrack {
        FullTrack {
            album: track.album.unwrap_or_default(),
            artists: track.artists,
            available_markets: track.available_markets.unwrap_or_default(),
            disc_number: track.disc_number,
            duration: track.duration,
            explicit: track.explicit,
            external_ids: HashMap::new(),
            external_urls: track.external_urls,
            href: track.href,
            id: track.id,
            is_local: track.is_local,
            is_playable: track.is_playable,
            linked_from: track.linked_from,
            restrictions: track.restrictions,
            name: track.name,
            popularity: 0,
            preview_url: track.preview_url,
            track_number: track.track_number,
        }
    }

    fn get_simple_album(&self, album: FullAlbum) -> SimplifiedAlbum {
        let album_type: &'static str = album.album_type.into();
        SimplifiedAlbum {
            album_group: None,
            album_type: Some(album_type.to_string()),
            artists: album.artists,
            available_markets: album.available_markets.unwrap_or_default(),
            external_urls: album.external_urls,
            href: Some(album.href),
            id: Some(album.id),
            images: album.images,
            name: album.name,
            release_date: Some(album.release_date),
            release_date_precision: None,
            restrictions: None,
        }
    }
}

#[async_trait]
impl GenericProvider for SpotifyProvider {
    #[tracing::instrument(level = "trace", skip(self))]
    async fn initialize(&mut self) -> Result<()> {
        let _ = self
            .status_tx
            .send(self.get_provider_status(None).await)
            .await;

        let preferences: State<PreferenceConfig> = self.app.state();
        let spotify_config: Value = preferences
            .load_selective("spotify".into())
            .unwrap_or_default();
        let client_id = spotify_config.get("client_id");
        let client_secret = spotify_config.get("client_secret");

        self.config.client_id = client_id.map(|v| v.as_str().unwrap().to_string());
        self.config.client_secret = client_secret.map(|v| v.as_str().unwrap().to_string());

        if self
            .config
            .client_id
            .as_ref()
            .map_or(true, |id| id.is_empty())
            || self
                .config
                .client_secret
                .as_ref()
                .map_or(true, |secret| secret.is_empty())
        {
            self.config.redirect_uri = "http://127.0.0.1:8898/login";
            self.config.client_id = Some("65b708073fc0480ea92a077233ca87bd".into())
        } else {
            self.config.redirect_uri = "https://moosync.app/spotify";
        }
        self.config.scopes = vec![
            "playlist-read-private",
            "user-top-read",
            "user-library-read",
            "user-read-private",
            "streaming",
            "app-remote-control",
        ];

        let res = self.refresh_login().await;
        if let Err(err) = res {
            tracing::error!("spotify refresh login err: {:?}", err);
        }

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn get_provider_scopes(&self) -> Result<Vec<ExtensionProviderScope>> {
        Ok(vec![
            ExtensionProviderScope::Search,
            ExtensionProviderScope::Playlists,
            ExtensionProviderScope::PlaylistSongs,
            ExtensionProviderScope::PlaybackDetails,
            ExtensionProviderScope::PlaylistFromUrl,
            ExtensionProviderScope::SearchAlbum,
            ExtensionProviderScope::SearchArtist,
            ExtensionProviderScope::Recommendations,
            ExtensionProviderScope::Accounts,
            ExtensionProviderScope::ArtistSongs,
            ExtensionProviderScope::AlbumSongs,
            ExtensionProviderScope::PlaylistSongs,
        ])
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn key(&self) -> String {
        "spotify".into()
    }

    #[tracing::instrument(level = "trace", skip(self, id))]
    fn match_id(&self, id: String) -> bool {
        id.starts_with("spotify-playlist:")
            || id.starts_with("spotify-artist:")
            || id.starts_with("spotify-album:")
            || id.starts_with("spotify:")
    }

    async fn requested_account_status(&mut self) -> Result<()> {
        // TODO: Get account status from spotify
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn login(&mut self, _: String) -> Result<String> {
        let (url, verifier) = login(
            LoginArgs {
                client_id: self.config.client_id.clone(),
                client_secret: self.config.client_secret.clone(),
                scopes: self.config.scopes.clone(),
                extra_params: None,
            },
            self.get_oauth_client(),
            &self.app,
        )?;
        self.verifier = verifier;

        let redirect_uri = self.config.redirect_uri;
        if redirect_uri.starts_with("http://127.0.0.1:8898") {
            let app_handle = self.app.clone();
            thread::spawn(move || {
                let socket_addr = Url::parse(redirect_uri)
                    .unwrap()
                    .socket_addrs(|| None)
                    .unwrap()
                    .pop()
                    .unwrap();

                tracing::info!("Listening {:?}", socket_addr);

                let listener = TcpListener::bind(socket_addr).unwrap();
                let stream = listener.incoming().flatten().next().unwrap();
                let mut reader = BufReader::new(&stream);
                let mut request_line = String::new();
                reader.read_line(&mut request_line).unwrap();

                let code = request_line.split_whitespace().nth(1);
                if let Some(code) = code {
                    tracing::info!("Got redirect URI {:?}", code);
                    let parsed_code = code.replace("/login", "");
                    let oauth_handler: State<OAuthHandler> = app_handle.state();
                    oauth_handler
                        .handle_oauth(
                            app_handle.clone(),
                            format!("moosync://spotifyoauthcallback{}", parsed_code),
                        )
                        .unwrap();
                }
            });
        }

        let oauth_handler: State<OAuthHandler> = self.app.state();
        oauth_handler.register_oauth_path("spotifyoauthcallback".into(), self.key());

        Ok(url)
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn signout(&mut self, _: String) -> Result<()> {
        self.config.tokens = None;
        self.api_client = None;
        self.verifier = None;

        let preferences: State<PreferenceConfig> = self.app.state();
        preferences.set_secure("MoosyncSpotifyRefreshToken".into(), None::<String>)?;

        let _ = self
            .status_tx
            .send(self.get_provider_status(None).await)
            .await;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self, code))]
    async fn authorize(&mut self, code: String) -> Result<()> {
        tracing::info!("Authorizing with code {}", code);
        self.config.tokens = Some(
            authorize(
                "MoosyncSpotifyRefreshToken",
                code,
                &mut self.verifier,
                &self.app,
            )
            .await?,
        );

        self.create_api_client().await;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self, pagination))]
    async fn fetch_user_playlists(
        &self,
        pagination: Pagination,
    ) -> Result<(Vec<QueryablePlaylist>, Pagination)> {
        let mut ret = vec![];
        tracing::info!("Fetching spotify playlists {:?}", self.api_client);
        if let Some(api_client) = &self.api_client {
            let playlists = api_client
                .current_user_playlists_manual(Some(pagination.limit), Some(pagination.offset))
                .await;
            if let Ok(playlists) = playlists {
                for playlist in playlists.items {
                    ret.push(self.parse_playlist(playlist))
                }
            }
            tracing::info!("Got user playlists {:?}", ret);
            return Ok((ret, pagination.next_page()));
        }

        Err("API client not initialized".into())
    }

    #[tracing::instrument(level = "trace", skip(self, playlist, pagination))]
    async fn get_playlist_content(
        &self,
        playlist: QueryablePlaylist,
        pagination: Pagination,
    ) -> Result<(Vec<Song>, Pagination)> {
        let mut ret = vec![];
        if playlist.playlist_id.is_none() {
            return Err("Playlist ID cannot be none".into());
        }
        let playlist_id = playlist.playlist_id.unwrap();
        if let Some(api_client) = &self.api_client {
            let playlist_id = playlist_id
                .strip_prefix("spotify-playlist:")
                .unwrap_or(&playlist_id);
            let items = api_client
                .playlist_items_manual(
                    PlaylistId::from_id_or_uri(playlist_id).unwrap(),
                    None,
                    None,
                    Some(pagination.limit),
                    Some(pagination.offset),
                )
                .await;
            if let Ok(items) = items {
                for i in items.items {
                    if i.is_local {
                        continue;
                    }

                    match i.track.unwrap() {
                        rspotify::model::PlayableItem::Track(t) => {
                            ret.push(self.parse_playlist_item(t));
                        }
                        rspotify::model::PlayableItem::Episode(_) => {
                            continue;
                        }
                    }
                }
            }
            return Ok((ret, pagination.next_page()));
        }
        Err("API client not initialized".into())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn get_playback_url(&self, _: Song, _: String) -> Result<String> {
        Err(MoosyncError::SwitchProviders("youtube".into()))
    }

    #[tracing::instrument(level = "trace", skip(self, term))]
    async fn search(&self, term: String) -> Result<SearchResult> {
        let mut ret = SearchResult {
            songs: vec![],
            albums: vec![],
            artists: vec![],
            playlists: vec![],
            ..Default::default()
        };

        if let Some(api_client) = &self.api_client {
            search_and_parse_all!(
                api_client,
                &term,
                [
                    (
                        SearchType::Track,
                        rspotify::model::SearchResult::Tracks,
                        |item, vec: &mut Vec<Song>| vec.push(self.parse_playlist_item(item)),
                        ret.songs
                    ),
                    (
                        SearchType::Playlist,
                        rspotify::model::SearchResult::Playlists,
                        |item, vec: &mut Vec<QueryablePlaylist>| vec
                            .push(self.parse_playlist(item)),
                        ret.playlists
                    ),
                    (
                        SearchType::Artist,
                        rspotify::model::SearchResult::Artists,
                        |item: FullArtist, vec: &mut Vec<QueryableArtist>| vec.push(
                            self.parse_artists(
                                SimplifiedArtist {
                                    external_urls: item.external_urls,
                                    href: Some(item.href),
                                    id: Some(item.id),
                                    name: item.name,
                                },
                                Some(item.images)
                            )
                        ),
                        ret.artists
                    ),
                    (
                        SearchType::Album,
                        rspotify::model::SearchResult::Albums,
                        |item, vec: &mut Vec<QueryableAlbum>| vec.push(self.parse_album(item)),
                        ret.albums
                    )
                ]
            );
            return Ok(ret);
        }
        Err("API client not initialized".into())
    }

    #[tracing::instrument(level = "trace", skip(self, url))]
    async fn match_url(&self, url: String) -> Result<bool> {
        let re = Regex::new(
            r"^(https:\/\/open.spotify.com\/(track|embed)\/|spotify:track:)([a-zA-Z0-9]+)(.*)$",
        )
        .unwrap();
        if re.is_match(url.as_str()) {
            return Ok(true);
        }

        let re = Regex::new(
            r"^(https:\/\/open.spotify.com\/playlist\/|spotify:playlist:)([a-zA-Z0-9]+)(.*)$",
        )
        .unwrap();
        if re.is_match(url.as_str()) {
            return Ok(true);
        }
        Ok(false)
    }

    #[tracing::instrument(level = "trace", skip(self, url))]
    async fn playlist_from_url(&self, url: String) -> Result<QueryablePlaylist> {
        let playlist_id = Url::parse(url.as_str());
        let playlist_id = if let Ok(playlist_id) = playlist_id {
            playlist_id.path().to_string()
        } else {
            url
        };

        if let Some(api_client) = &self.api_client {
            let playlists = api_client
                .playlist(
                    PlaylistId::from_id_or_uri(playlist_id.as_str())
                        .map_err(|_| MoosyncError::String("Invalid playlist url".into()))?,
                    None,
                    None,
                )
                .await?;

            let res = self.parse_playlist(SimplifiedPlaylist {
                collaborative: playlists.collaborative,
                external_urls: playlists.external_urls,
                href: playlists.href,
                id: playlists.id,
                images: playlists.images,
                name: playlists.name,
                owner: playlists.owner,
                public: playlists.public,
                snapshot_id: playlists.snapshot_id,
                tracks: PlaylistTracksRef::default(),
            });

            return Ok(res);
        }

        Err("API Client not initialized".into())
    }

    #[tracing::instrument(level = "trace", skip(self, url))]
    async fn song_from_url(&self, url: String) -> Result<Song> {
        let track_id = Url::parse(url.as_str());
        let track_id = if let Ok(track_id) = track_id {
            track_id.path().to_string()
        } else {
            url
        };

        if let Some(api_client) = &self.api_client {
            let res = api_client
                .track(TrackId::from_id_or_uri(track_id.as_str())?, None)
                .await?;

            return Ok(self.parse_playlist_item(res));
        }

        Err("API Client not initialized".into())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn get_suggestions(&self) -> Result<Vec<Song>> {
        if let Some(api_client) = &self.api_client {
            let mut i = 0;
            let mut ret = vec![];
            while i < 5 {
                let user_top_tracks = api_client.current_user_top_tracks(None).next().await;
                if let Some(track) = user_top_tracks {
                    let track = track?;
                    if let Some(track_id) = track.id {
                        ret.push(track_id);
                        i += 1;
                    }
                } else {
                    break;
                }
            }
            let recom = api_client
                .recommendations(
                    vec![],
                    Some(vec![]),
                    Some(vec![]),
                    Some(ret),
                    None,
                    Some(100),
                )
                .await?;
            return Ok(recom
                .tracks
                .iter()
                .map(|t| {
                    self.parse_playlist_item(FullTrack {
                        album: t.album.clone().unwrap_or_default(),
                        artists: t.artists.clone(),
                        disc_number: t.disc_number,
                        duration: t.duration,
                        id: t.id.clone(),
                        name: t.name.clone(),
                        track_number: t.track_number,
                        available_markets: vec![],
                        explicit: t.explicit,
                        external_urls: t.external_urls.clone(),
                        external_ids: HashMap::new(),
                        href: None,
                        is_local: false,
                        is_playable: None,
                        linked_from: None,
                        restrictions: None,
                        popularity: 0,
                        preview_url: t.preview_url.clone(),
                    })
                })
                .collect());
        }
        Err("API Client not initialized".into())
    }

    async fn get_album_content(
        &self,
        album: QueryableAlbum,
        pagination: Pagination,
    ) -> Result<(Vec<Song>, Pagination)> {
        if let Some(api_client) = &self.api_client {
            let mut raw_id = album.album_id;
            if let Some(id) = &raw_id {
                if !self.match_id(id.clone()) {
                    if let Some(album_name) = album.album_name {
                        let res = self.search(album_name).await?;
                        if let Some(album) = res.albums.first() {
                            raw_id = album.album_id.clone();
                        } else {
                            raw_id = None;
                        }
                    } else {
                        raw_id = None;
                    }
                }
            }

            if let Some(id) = &raw_id {
                tracing::debug!("Got album id: {}", id);
                let id = id.replace("spotify-album:", "");
                let id = AlbumId::from_id_or_uri(id.as_str())?;
                let album = api_client.album(id.clone(), None).await?;
                let album_tracks = api_client
                    .album_track_manual(id, None, Some(pagination.limit), Some(pagination.offset))
                    .await?;
                let mut items = album_tracks.items.clone();
                let songs = items
                    .iter_mut()
                    .map(|t| {
                        t.album = Some(self.get_simple_album(album.clone()));
                        self.parse_playlist_item(self.get_full_track(t.clone()))
                    })
                    .collect::<Vec<_>>();

                return Ok((songs, pagination.next_page()));
            }
        }
        Err("API Client not initialized".into())
    }

    async fn get_artist_content(
        &self,
        artist: QueryableArtist,
        pagination: Pagination,
    ) -> Result<(Vec<Song>, Pagination)> {
        if let Some(api_client) = &self.api_client {
            if let Some(next_page_token) = &pagination.token {
                // TODO: Fetch next pages
                let _tokens = next_page_token.split(";").collect::<Vec<_>>();
                return Ok((vec![], pagination.next_page_wtoken(None)));
            }

            let mut raw_id = artist.artist_id;
            if let Some(id) = &raw_id {
                if !self.match_id(id.clone()) {
                    if let Some(artist_name) = artist.artist_name {
                        let res = self.search(artist_name).await?;
                        if let Some(artist) = res.artists.first() {
                            raw_id = artist.artist_id.clone();
                        } else {
                            raw_id = None;
                        }
                    } else {
                        raw_id = None;
                    }
                }
            }

            if let Some(id) = &raw_id {
                tracing::debug!("Got artist id: {}", id);
                let mut songs = vec![];
                let mut next_page_tokens = vec![];
                let id = id.replace("spotify-artist:", "");
                let albums =
                    api_client.artist_albums(ArtistId::from_id_or_uri(id.as_str())?, [], None);

                let album_ids = albums.filter_map(|a| async {
                    if let Ok(a) = a {
                        if let Some(id) = a.id {
                            return Some(id);
                        }
                    }
                    None
                });

                let album_ids = album_ids.collect::<Vec<_>>().await;

                for chunk in album_ids.chunks(20) {
                    let albums = api_client.albums(chunk.to_vec(), None).await?;
                    tracing::debug!("Got albums {:?}", albums);
                    for a in albums {
                        let mut tracks = a.tracks.items.clone();
                        let parsed = tracks.iter_mut().map(|t| {
                            t.album = Some(self.get_simple_album(a.clone()));
                            self.parse_playlist_item(self.get_full_track(t.clone()))
                        });

                        songs.extend(parsed);

                        if let Some(next) = a.tracks.next {
                            next_page_tokens.push(next);
                        }
                    }
                }

                let next_page_token = next_page_tokens.join(";");
                return Ok((songs, pagination.next_page_wtoken(Some(next_page_token))));
            }
        }
        Err("API Client not initialized".into())
    }

    async fn get_lyrics(&self, _: Song) -> Result<String> {
        return Err("Not implemented".into());
    }

    async fn get_song_context_menu(&self, _: Vec<Song>) -> Result<Vec<ContextMenuReturnType>> {
        return Err("Not implemented".into());
    }

    async fn get_playlist_context_menu(
        &self,
        _: QueryablePlaylist,
    ) -> Result<Vec<ContextMenuReturnType>> {
        return Err("Not implemented".into());
    }

    async fn trigger_context_menu_action(&self, _: String) -> Result<()> {
        return Err("Not implemented".into());
    }
}
