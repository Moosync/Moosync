use std::collections::HashSet;

use async_trait::async_trait;

use chrono::{DateTime, TimeDelta};
use oauth2::{CsrfToken, PkceCodeVerifier, TokenResponse};
use preferences::preferences::PreferenceConfig;
use rspotify::{
    clients::{BaseClient, OAuthClient},
    model::{
        FullArtist, FullTrack, Id, PlaylistId, SearchType, SimplifiedAlbum, SimplifiedArtist,
        SimplifiedPlaylist,
    },
    AuthCodePkceSpotify, Token,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager, State};
use types::{
    entities::{EntityInfo, QueryableAlbum, QueryableArtist, QueryablePlaylist, SearchResult},
    errors::errors::Result,
    oauth::OAuth2Client,
    providers::generic::{Pagination, ProviderStatus},
    songs::{QueryableSong, Song, SongType},
};
use types::{errors::errors::MoosyncError, providers::generic::GenericProvider};

use super::common::{
    authorize, get_oauth_client, login, refresh_login, LoginArgs, OAuthClientArgs, TokenHolder,
};

macro_rules! search_and_parse_all {
    ($client:expr, $term:expr, [$(($type:expr, $variant:path, $parse_fn:expr, $result_vec:expr)),*]) => {{
        $(
            if let Ok(search_results) = $client.search($term, $type, None, None, Some(50), Some(0)).await {
                if let $variant(items) = search_results {
                    for item in items.items {
                        $parse_fn(item, &mut $result_vec);
                    }
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
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            config: SpotifyConfig::default(),
            verifier: None,
            api_client: None,
        }
    }
}

impl SpotifyProvider {
    fn get_oauth_client(&self) -> OAuth2Client {
        get_oauth_client(OAuthClientArgs {
            auth_url: "https://accounts.spotify.com/authorize".to_string(),
            token_url: "https://accounts.spotify.com/api/token".to_string(),
            redirect_url: self.config.redirect_uri.to_string(),
            client_id: self.config.client_id.clone().unwrap(),
            client_secret: self.config.client_secret.clone().unwrap(),
        })
    }

    fn create_api_client(&mut self) {
        println!("Creating api client");
        if let Some(token) = &self.config.tokens {
            self.api_client = Some(AuthCodePkceSpotify::from_token(Token {
                access_token: token.access_token.clone(),
                expires_in: TimeDelta::seconds(token.expires_in.try_into().unwrap()),
                expires_at: Some(DateTime::from_timestamp_millis(token.expires_at).unwrap()),
                refresh_token: Some(token.refresh_token.clone()),
                scopes: HashSet::from_iter(self.config.scopes.iter().map(|v| v.to_string())),
            }));
        }
    }

    async fn refresh_login(&mut self) -> Result<()> {
        self.config.tokens = Some(
            refresh_login(
                "MoosyncSpotifyRefreshToken",
                self.get_oauth_client(),
                &self.app,
            )
            .await?,
        );
        self.create_api_client();

        Ok(())
    }

    fn parse_playlist(&self, playlist: SimplifiedPlaylist) -> QueryablePlaylist {
        QueryablePlaylist {
            playlist_id: Some(format!("spotify-playlist:{}", playlist.id.id())),
            playlist_name: playlist.name,
            playlist_coverpath: playlist.images.first().map(|i| i.url.clone()),
            playlist_song_count: playlist.tracks.total as f64,
            ..Default::default()
        }
    }

    fn parse_artists(&self, artist: SimplifiedArtist) -> QueryableArtist {
        QueryableArtist {
            artist_id: Some(format!("spotify-artist:{}", artist.id.clone().unwrap())),
            artist_name: Some(artist.name),
            artist_extra_info: Some(EntityInfo(
                serde_json::to_value(SpotifyExtraInfo {
                    spotify: ArtistExtraInfo {
                        artist_id: artist.id.clone().unwrap().to_string(),
                    },
                })
                .unwrap(),
            )),
            ..Default::default()
        }
    }

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
                ..Default::default()
            },
            album: Some(self.parse_album(item.album)),
            artists: Some(
                item.artists
                    .into_iter()
                    .map(|a| self.parse_artists(a))
                    .collect(),
            ),
            ..Default::default()
        }
    }
}

#[async_trait]
impl GenericProvider for SpotifyProvider {
    async fn initialize(&mut self) -> Result<()> {
        let preferences: State<PreferenceConfig> = self.app.state();
        let spotify_config: Value = preferences.inner().load_selective("spotify".into())?;
        println!("{:?}", spotify_config);
        let client_id = spotify_config.get("client_id");
        let client_secret = spotify_config.get("client_secret");

        self.config.client_id = client_id.map(|v| v.as_str().unwrap().to_string());
        self.config.client_secret = client_secret.map(|v| v.as_str().unwrap().to_string());
        self.config.redirect_uri = "https://moosync.app/spotify";
        self.config.scopes = vec![
            "playlist-read-private",
            "user-top-read",
            "user-library-read",
            "user-read-private",
        ];

        let res = self.refresh_login().await;
        if let Err(err) = res {
            println!("spotify refresh login err: {:?}", err);
        }

        println!("initialized {:?}", self.config);

        Ok(())
    }

    fn key(&self) -> &str {
        "spotify"
    }

    fn match_id(&self, id: String) -> bool {
        id.starts_with("spotify-playlist:")
            || id.starts_with("spotify-artist:")
            || id.starts_with("spotify-album:")
            || id.starts_with("spotify:")
    }

    async fn login(&mut self) -> Result<()> {
        self.verifier = login(
            LoginArgs {
                client_id: self.config.client_id.clone(),
                client_secret: self.config.client_secret.clone(),
                scopes: self.config.scopes.clone(),
                extra_params: None,
            },
            self.get_oauth_client(),
            &self.app,
        )?;

        Ok(())
    }

    async fn authorize(&mut self, code: String) -> Result<()> {
        self.config.tokens = Some(
            authorize(
                "MoosyncSpotifyRefreshToken",
                code,
                &mut self.verifier,
                &self.app,
            )
            .await?,
        );

        self.create_api_client();
        Ok(())
    }

    async fn fetch_user_details(&self) -> Result<ProviderStatus> {
        println!("Fetchinf user details {:?}", self.api_client);
        if let Some(api_client) = &self.api_client {
            let token = api_client.token.lock().await.unwrap();
            println!("tokens {:?}", token.clone().unwrap().is_expired());
            drop(token);

            let user = api_client.current_user().await?;
            return Ok(ProviderStatus {
                key: self.key().into(),
                name: "Spotify".into(),
                user_name: user.display_name,
                logged_in: true,
            });
        }

        Err("API client not initialized".into())
    }

    async fn fetch_user_playlists(
        &self,
        pagination: Pagination,
    ) -> Result<(Vec<QueryablePlaylist>, Pagination)> {
        let mut ret = vec![];
        if let Some(api_client) = &self.api_client {
            let playlists = api_client
                .current_user_playlists_manual(Some(pagination.limit), Some(pagination.offset))
                .await;
            if let Ok(playlists) = playlists {
                for playlist in playlists.items {
                    ret.push(self.parse_playlist(playlist))
                }
            }
        }
        Ok((ret, pagination.next_page()))
    }

    async fn get_playlist_content(
        &self,
        playlist_id: String,
        pagination: Pagination,
    ) -> Result<(Vec<Song>, Pagination)> {
        let mut ret = vec![];
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
        }
        Ok((ret, pagination.next_page()))
    }

    async fn get_playback_url(&self, _: Song, _: String) -> Result<String> {
        Err(MoosyncError::SwitchProviders("youtube".into()))
    }

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
                            self.parse_artists(SimplifiedArtist {
                                external_urls: item.external_urls,
                                href: Some(item.href),
                                id: Some(item.id),
                                name: item.name,
                            })
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
        }

        Ok(ret)
    }
}
