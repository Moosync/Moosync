use std::{
    collections::HashSet,
    time::{SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;

use chrono::{DateTime, TimeDelta};
use oauth2::{
    basic::{BasicClient, BasicErrorResponseType, BasicTokenType},
    reqwest::async_http_client,
    AuthUrl, AuthorizationCode, Client, ClientId, ClientSecret, CsrfToken, EmptyExtraTokenFields,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, RefreshToken, RevocationErrorResponseType,
    Scope, StandardErrorResponse, StandardRevocableToken, StandardTokenIntrospectionResponse,
    StandardTokenResponse, TokenResponse, TokenUrl,
};
use preferences::preferences::PreferenceConfig;
use rspotify::{
    clients::{BaseClient, OAuthClient},
    model::{FullTrack, Id, PlaylistId, SimplifiedPlaylist},
    AuthCodePkceSpotify, Token,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager, State};
use types::{
    entities::{EntityInfo, QueryableAlbum, QueryableArtist, QueryablePlaylist},
    errors::errors::Result,
    oauth::{OAuth2Client, OAuthTokenResponse},
    providers::generic::{Pagination, ProviderStatus},
    songs::{QueryableSong, Song, SongType},
};
use types::{errors::errors::MoosyncError, providers::generic::GenericProvider};
use url::Url;

use crate::window::handler::WindowHandler;

use super::common::{
    authorize, get_oauth_client, login, refresh_login, set_tokens, LoginArgs, OAuthClientArgs,
    TokenHolder,
};

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
            album: Some(QueryableAlbum {
                album_id: Some(format!("spotify-album:{}", item.album.id.unwrap())),
                album_name: Some(item.album.name),
                album_artist: item.album.artists.first().map(|a| a.name.clone()),
                album_coverpath_high: item.album.images.first().map(|i| i.url.clone()),
                ..Default::default()
            }),
            artists: Some(
                item.artists
                    .into_iter()
                    .map(|artist| QueryableArtist {
                        artist_id: Some(format!("spotify-artist:{}", artist.id.clone().unwrap())),
                        artist_name: Some(artist.name.clone()),
                        artist_extra_info: Some(EntityInfo(
                            serde_json::to_value(SpotifyExtraInfo {
                                spotify: ArtistExtraInfo {
                                    artist_id: artist.id.unwrap().to_string(),
                                },
                            })
                            .unwrap(),
                        )),
                        sanitized_artist_name: Some(artist.name),
                        ..Default::default()
                    })
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
        let spotify_config = preferences.inner().load_selective("spotify".into())?;
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
}
