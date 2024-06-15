use futures::StreamExt;
use std::{collections::HashSet};
use web_time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;

use chrono::{DateTime, TimeDelta};
use leptos::{create_rw_signal, RwSignal, SignalUpdate};
use oauth2::{
    basic::{BasicClient, BasicErrorResponseType, BasicTokenType},
    reqwest::async_http_client,
    AccessToken, AuthUrl, AuthorizationCode, Client, ClientId, ClientSecret, CsrfToken,
    EmptyExtraTokenFields, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, RefreshToken,
    RevocationErrorResponseType, Scope, StandardErrorResponse, StandardRevocableToken,
    StandardTokenIntrospectionResponse, StandardTokenResponse, TokenResponse, TokenUrl,
};
use rspotify::{
    clients::{BaseClient, OAuthClient},
    model::{FullTrack, Id, PlaylistId, SimplifiedPlaylist},
    AuthCodePkceSpotify, Token,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use types::{
    entities::{EntityInfo, QueryableAlbum, QueryableArtist, QueryablePlaylist},
    errors::errors::Result,
    songs::{QueryableSong, Song, SongType},
    ui::providers::ProviderStatus,
};
use url::Url;

use crate::{
    console_log,
    utils::{
        prefs::{get_secure_async, load_selective_async, set_secure_async},
        window::open_external,
    },
};

use super::generic::GenericProvider;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenHolder {
    access_token: String,
    refresh_token: String,
    expires_in: u64,
    expires_at: i64,
}

#[derive(Debug, Clone, Default)]
struct SpotifyConfig {
    client_secret: Option<String>,
    client_id: Option<String>,
    redirect_uri: &'static str,
    scopes: Vec<&'static str>,
    tokens: Option<TokenHolder>,
}

type OAuthTokenResponse = StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>;
type OAuth2Client = Client<
    StandardErrorResponse<BasicErrorResponseType>,
    OAuthTokenResponse,
    BasicTokenType,
    StandardTokenIntrospectionResponse<EmptyExtraTokenFields, BasicTokenType>,
    StandardRevocableToken,
    StandardErrorResponse<RevocationErrorResponseType>,
>;

#[derive(Debug, Default)]
pub struct SpotifyProvider {
    config: SpotifyConfig,
    status: RwSignal<ProviderStatus>,
    verifier: Option<(OAuth2Client, PkceCodeVerifier, CsrfToken)>,
    api_client: Option<AuthCodePkceSpotify>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ArtistExtraInfo {
    artist_id: String
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SpotifyExtraInfo {
    spotify: ArtistExtraInfo
}

impl SpotifyProvider {
    pub fn new() -> Self {
        Self {
            config: SpotifyConfig::default(),
            status: create_rw_signal(ProviderStatus::with_name("Spotify")),
            verifier: None,
            api_client: None,
        }
    }
}

impl SpotifyProvider {
    fn get_oauth_client(&self) -> OAuth2Client {
        BasicClient::new(
            ClientId::new(self.config.client_id.clone().unwrap()),
            Some(ClientSecret::new(
                self.config.client_secret.clone().unwrap(),
            )),
            AuthUrl::new("https://accounts.spotify.com/authorize".to_string()).unwrap(),
            Some(TokenUrl::new("https://accounts.spotify.com/api/token".to_string()).unwrap()),
        )
        .set_redirect_uri(RedirectUrl::new(self.config.redirect_uri.to_string()).unwrap())
    }

    fn create_api_client(&mut self) {
        console_log!("Creating api client");
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

    async fn set_tokens(&mut self, res: OAuthTokenResponse) {
        console_log!("{:?}", res.extra_fields());

        let expires_in = res.expires_in().unwrap_or_default();
        self.config.tokens = Some(TokenHolder {
            access_token: res.access_token().secret().clone(),
            refresh_token: res.refresh_token().unwrap().secret().clone(),
            expires_in: expires_in.as_secs(),
            expires_at: (SystemTime::now().duration_since(UNIX_EPOCH).unwrap() + expires_in)
                .as_millis() as i64,
        });

        set_secure_async(
            "MoosyncSpotifyRefreshToken",
            Value::String(res.refresh_token().unwrap().secret().clone()),
        )
        .await
        .unwrap();

        if cfg!(feature = "mock") {
            set_secure_async(
                "MoosyncSpotifyAccessToken",
                serde_json::to_value(self.config.tokens.clone().unwrap()).unwrap(),
            )
            .await
            .unwrap();
        }

        self.create_api_client();
    }

    async fn refresh_login(&mut self) -> Result<()> {
        let refresh_token = get_secure_async("MoosyncSpotifyRefreshToken").await?;
        if !refresh_token.is_null() {
            let refresh_token = refresh_token.as_str().unwrap();

            if cfg!(feature = "mock") {
                let token_holder: TokenHolder =
                    serde_json::from_value(get_secure_async("MoosyncSpotifyAccessToken").await?)?;
                let mut res = OAuthTokenResponse::new(
                    AccessToken::new(token_holder.access_token),
                    BasicTokenType::Bearer,
                    EmptyExtraTokenFields {},
                );
                res.set_refresh_token(Some(RefreshToken::new(token_holder.refresh_token)));
                res.set_expires_in(Some(&Duration::from_secs(token_holder.expires_in)));

                self.set_tokens(res).await;
                return Ok(());
            }

            let client = self.get_oauth_client();
            let res = client
                .exchange_refresh_token(&RefreshToken::new(refresh_token.to_string()))
                .request_async(async_http_client)
                .await?;

            self.set_tokens(res).await;
            return Ok(());
        }
        Err("Refresh token not found".into())
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
            artists: Some(item.artists.into_iter().map(|artist| {
                QueryableArtist {
                    artist_id: Some(format!("spotify-artist:{}", artist.id.clone().unwrap())),
                    artist_name: Some(artist.name.clone()),
                    artist_extra_info: Some(EntityInfo(serde_json::to_value(SpotifyExtraInfo {
                        spotify: ArtistExtraInfo { artist_id: artist.id.unwrap().to_string() }
                    }).unwrap())),
                    sanitized_artist_name: Some(artist.name),
                    ..Default::default()
                }
            }).collect()),
            ..Default::default()
        }
    }
}

#[async_trait(?Send)]
impl GenericProvider for SpotifyProvider {
    async fn initialize(&mut self) -> Result<()> {
        let spotify_config = load_selective_async("spotify").await?;
        console_log!("{:?}", spotify_config);
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
            console_log!("spotify refresh login err: {:?}", err);
        }

        console_log!("initialized {:?}", self.config);

        Ok(())
    }

    fn key(&self) -> &str {
        "spotify"
    }

    fn get_status(&self) -> RwSignal<ProviderStatus> {
        self.status
    }

    fn match_id(&self, id: String) -> bool {
        return id.starts_with("spotify-playlist:") || id.starts_with("spotify-artist:") || id.starts_with("spotify-album:") || id.starts_with("spotify:")
    }

    async fn login(&mut self) -> Result<()> {
        if self.config.client_id.is_none() || self.config.client_secret.is_none() {
            return Err("Client ID not set".into());
        }

        let client = self.get_oauth_client();

        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (auth_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .add_scopes(self.config.scopes.iter().map(|s| Scope::new(s.to_string())))
            .set_pkce_challenge(pkce_challenge)
            .url();

        self.verifier = Some((client, pkce_verifier, csrf_token.clone()));
        open_external(auth_url.to_string()).await;

        Ok(())
    }

    async fn authorize(&mut self, code: String) -> Result<()> {
        if self.verifier.is_none() {
            return Err("OAuth not initiated".into());
        }

        let parsed_code = Url::parse(format!("https://moosync.app/{}", code).as_str()).unwrap();
        let code = parsed_code
            .query_pairs()
            .find(|p| p.0 == "code")
            .unwrap()
            .1
            .to_string();

        let (client, verifier, csrf) = self.verifier.take().unwrap();

        let res = client
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(verifier)
            .request_async(async_http_client)
            .await?;
        self.set_tokens(res).await;
        Ok(())
    }

    async fn fetch_user_details(&self) -> Result<()> {
        console_log!("Fetchinf user details {:?}", self.api_client);
        if let Some(api_client) = &self.api_client {
            let token = api_client.token.lock().await.unwrap();
            console_log!("tokens {:?}", token.clone().unwrap().is_expired());
            drop(token);

            let user = api_client.current_user().await?;
            self.status.update(|s| {
                s.user_name = user.display_name;
                s.logged_in = true;
            });
        }
        Ok(())
    }

    async fn fetch_user_playlists(
        &self,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<QueryablePlaylist>> {
        let mut ret = vec![];
        if let Some(api_client) = &self.api_client {
            let playlists = api_client
                .current_user_playlists_manual(Some(limit), Some(offset))
                .await;
            if let Ok(playlists) = playlists {
                for playlist in playlists.items {
                    ret.push(self.parse_playlist(playlist))
                }
            }
        }
        Ok(ret)
    }

    async fn get_playlist_content(
        &self,
        playlist_id: String,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<Song>> {
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
                    Some(limit),
                    Some(offset),
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
        Ok(ret)
    }
}
