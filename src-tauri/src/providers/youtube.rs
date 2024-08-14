use async_trait::async_trait;
use futures::channel::mpsc::UnboundedSender;
use futures::SinkExt;
use google_youtube3::api::{Channel, ChannelSnippet, Playlist, PlaylistSnippet, Video};
use google_youtube3::hyper::client::HttpConnector;
use google_youtube3::hyper_rustls::HttpsConnector;
use google_youtube3::{hyper, hyper_rustls, YouTube};
use oauth2::basic::BasicClient;
use oauth2::{
    AuthUrl, ClientId, ClientSecret, CsrfToken, PkceCodeVerifier, RedirectUrl, TokenResponse,
    TokenUrl,
};
use preferences::preferences::PreferenceConfig;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use tauri::{AppHandle, Manager, State};
use types::entities::{
    EntityInfo, QueryableAlbum, QueryableArtist, QueryablePlaylist, SearchResult,
};
use types::errors::errors::{MoosyncError, Result};
use types::providers::generic::{Pagination, ProviderStatus};
use types::songs::{QueryableSong, Song, SongType};
use types::{oauth::OAuth2Client, providers::generic::GenericProvider};
use youtube::types::ContinuationToken;
use youtube::youtube::YoutubeScraper;

use super::common::{authorize, login, refresh_login, LoginArgs, TokenHolder};

macro_rules! search_and_parse {
    ($client:expr, $term:expr, $type:expr, $process_fn:expr) => {{
        let (_, search_results) = $client
            .search()
            .list(&vec!["snippet".into()])
            .add_type($type)
            .q($term)
            .max_results(50)
            .doit()
            .await?;

        search_results.items.map_or(vec![], |items| {
            items.into_iter().filter_map($process_fn).collect()
        })
    }};
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ArtistExtraInfo {
    artist_id: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct YoutubeExtraInfo {
    youtube: ArtistExtraInfo,
}

#[derive(Debug, Clone, Default)]
struct YoutubeConfig {
    client_secret: Option<String>,
    client_id: Option<String>,
    redirect_uri: &'static str,
    scopes: Vec<&'static str>,
    tokens: Option<TokenHolder>,
}

pub struct YoutubeProvider {
    app: AppHandle,
    config: YoutubeConfig,
    verifier: Option<(OAuth2Client, PkceCodeVerifier, CsrfToken)>,
    api_client: Option<YouTube<HttpsConnector<HttpConnector>>>,
    status_tx: UnboundedSender<ProviderStatus>,
}

impl std::fmt::Debug for YoutubeProvider {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        <YoutubeConfig as std::fmt::Debug>::fmt(&self.config, f)
    }
}

impl YoutubeProvider {
    pub fn new(app: AppHandle, status_tx: UnboundedSender<ProviderStatus>) -> Self {
        Self {
            app,
            config: YoutubeConfig::default(),
            verifier: None,
            api_client: None,
            status_tx,
        }
    }

    fn get_oauth_client(&self) -> OAuth2Client {
        BasicClient::new(
            ClientId::new(self.config.client_id.clone().unwrap()),
            Some(ClientSecret::new(
                self.config.client_secret.clone().unwrap(),
            )),
            AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string()).unwrap(),
            Some(TokenUrl::new("https://oauth2.googleapis.com/token".to_string()).unwrap()),
        )
        .set_redirect_uri(RedirectUrl::new(self.config.redirect_uri.to_string()).unwrap())
    }

    async fn create_api_client(&mut self) {
        if let Some(token) = &self.config.tokens {
            let client = hyper::Client::builder().build(
                hyper_rustls::HttpsConnectorBuilder::new()
                    .with_native_roots()
                    .unwrap()
                    .https_or_http()
                    .enable_http1()
                    .build(),
            );

            self.api_client = Some(google_youtube3::YouTube::new(
                client,
                token.access_token.clone(),
            ));

            let res = self.fetch_user_details().await;
            if let Ok(res) = res {
                let _ = self.status_tx.send(res).await;
            } else {
                let _ = self
                    .status_tx
                    .send(ProviderStatus {
                        key: self.key(),
                        name: "Youtube".into(),
                        user_name: None,
                        logged_in: true,
                        bg_color: "#E62017".into(),
                    })
                    .await;
            }
        }
    }

    async fn refresh_login(&mut self) -> Result<()> {
        self.config.tokens = Some(
            refresh_login(
                "MoosyncYoutubeRefreshToken",
                self.get_oauth_client(),
                &self.app,
            )
            .await?,
        );
        self.create_api_client().await;

        Ok(())
    }

    fn parse_playlist(&self, resp: Playlist) -> QueryablePlaylist {
        let snippet = resp.snippet.unwrap_or_default();
        let content_details = resp.content_details.unwrap_or_default();

        QueryablePlaylist {
            playlist_id: Some(format!("youtube-playlist:{}", resp.id.unwrap())),
            playlist_name: snippet.title.unwrap_or_default(),
            playlist_coverpath: snippet.thumbnails.map(|t| {
                t.maxres
                    .or(t.default)
                    .unwrap_or_default()
                    .url
                    .unwrap_or_default()
            }),
            playlist_song_count: content_details.item_count.unwrap_or_default() as f64,
            playlist_desc: snippet.description,
            playlist_path: None,
            extension: None,
            icon: None,
        }
    }

    fn parse_channel(&self, resp: Channel) -> QueryableArtist {
        let snippet = resp.snippet.as_ref().unwrap();
        QueryableArtist {
            artist_id: Some(format!("youtube-artist:{}", resp.id.clone().unwrap())),
            artist_name: snippet.title.clone(),
            artist_coverpath: snippet.thumbnails.clone().map(|t| {
                t.maxres
                    .or(t.default)
                    .unwrap_or_default()
                    .url
                    .unwrap_or_default()
            }),
            artist_extra_info: Some(EntityInfo(
                serde_json::to_value(YoutubeExtraInfo {
                    youtube: ArtistExtraInfo {
                        artist_id: resp.id.unwrap(),
                    },
                })
                .unwrap(),
            )),
            ..Default::default()
        }
    }

    async fn fetch_song_details(&self, ids: Vec<String>) -> Result<Vec<Song>> {
        println!("Fetching song details for {:?}", ids);
        if let Some(api_client) = &self.api_client {
            let mut ret = vec![];

            for id_chunk in ids.chunks(50) {
                let mut builder = api_client
                    .videos()
                    .list(&vec!["contentDetails".into(), "snippet".into()]);
                for i in id_chunk {
                    builder = builder.add_id(i);
                }

                let (_, resp) = builder.doit().await?;
                println!("Got song response {:?}", resp);
                if let Some(videos) = resp.items {
                    for v in videos {
                        ret.push(self.parse_video_item(v));
                    }
                }
            }

            return Ok(ret);
        }

        Err("API client not initialized".into())
    }

    fn parse_video_item(&self, resp: Video) -> Song {
        let snippet = resp.snippet.unwrap_or_default();
        let content_details = resp.content_details.unwrap_or_default();
        let id = resp.id;

        Song {
            song: QueryableSong {
                _id: id.clone().map(|id| format!("youtube:{}", id)),
                title: snippet.title,
                date: snippet.published_at.map(|v| v.to_string()),
                duration: content_details.duration.map(|d| {
                    core::time::Duration::from(iso8601::duration(&d).unwrap()).as_secs() as f64
                }),
                type_: SongType::YOUTUBE,
                url: id.clone(),
                song_cover_path_high: snippet
                    .thumbnails
                    .clone()
                    .map(|t| t.maxres.unwrap_or_default().url.unwrap_or_default()),
                playback_url: id,
                song_cover_path_low: snippet
                    .thumbnails
                    .map(|t| t.standard.unwrap_or_default().url.unwrap_or_default()),
                date_added: snippet.published_at.map(|v| v.timestamp_millis()),
                ..Default::default()
            },
            album: Some(QueryableAlbum {
                album_name: Some("Misc".into()),
                ..Default::default()
            }),
            artists: Some(vec![QueryableArtist {
                artist_id: snippet
                    .channel_id
                    .map(|id| format!("youtube-artist:{}", id)),
                artist_name: snippet.channel_title,
                ..Default::default()
            }]),
            genre: None,
        }
    }

    async fn fetch_user_details(&self) -> Result<ProviderStatus> {
        if let Some(api_client) = &self.api_client {
            let (_, user_info) = api_client
                .channels()
                .list(&vec!["snippet".into()])
                .mine(true)
                .max_results(1)
                .doit()
                .await?;

            let mut username = Some("".to_string());
            if let Some(items) = user_info.items {
                let channel = items.first().unwrap();
                if let Some(snippet) = &channel.snippet {
                    username = snippet.title.clone();
                }
            }
            return Ok(ProviderStatus {
                key: self.key(),
                name: "Youtube".into(),
                user_name: username,
                logged_in: true,
                bg_color: "#E62017".into(),
            });
        }

        Err("API client not initialized".into())
    }
}

#[async_trait]
impl GenericProvider for YoutubeProvider {
    async fn initialize(&mut self) -> Result<()> {
        let _ = self
            .status_tx
            .send(ProviderStatus {
                key: self.key(),
                name: "Youtube".into(),
                user_name: None,
                logged_in: false,
                bg_color: "#E62017".into(),
            })
            .await;

        let preferences: State<PreferenceConfig> = self.app.state();
        let youtube_config: Value = preferences.inner().load_selective("youtube".into())?;
        println!("{:?}", youtube_config);
        let client_id = youtube_config.get("client_id");
        let client_secret = youtube_config.get("client_secret");

        self.config.client_id = client_id.map(|v| v.as_str().unwrap().to_string());
        self.config.client_secret = client_secret.map(|v| v.as_str().unwrap().to_string());
        self.config.redirect_uri = "https://moosync.app/youtube";
        self.config.scopes = vec!["https://www.googleapis.com/auth/youtube.readonly"];

        let res = self.refresh_login().await;
        if let Err(err) = res {
            println!("youtube refresh login err: {:?}", err);
        }

        Ok(())
    }

    fn key(&self) -> String {
        "youtube".into()
    }

    fn match_id(&self, id: String) -> bool {
        id.starts_with("youtube-playlist:")
            || id.starts_with("youtube-artist:")
            || id.starts_with("youtube-album:")
            || id.starts_with("youtube:")
    }

    async fn login(&mut self) -> Result<()> {
        self.verifier = login(
            LoginArgs {
                client_id: self.config.client_id.clone(),
                client_secret: self.config.client_secret.clone(),
                scopes: self.config.scopes.clone(),
                extra_params: Some(HashMap::from([
                    ("prompt", "consent"),
                    ("access_type", "offline"),
                ])),
            },
            self.get_oauth_client(),
            &self.app,
        )?;

        Ok(())
    }

    async fn authorize(&mut self, code: String) -> Result<()> {
        self.config.tokens = Some(
            authorize(
                "MoosyncYoutubeRefreshToken",
                code,
                &mut self.verifier,
                &self.app,
            )
            .await?,
        );

        self.create_api_client().await;

        // Remove
        self.fetch_user_details().await.unwrap();
        Ok(())
    }

    async fn fetch_user_playlists(
        &self,
        pagination: Pagination,
    ) -> Result<(Vec<QueryablePlaylist>, Pagination)> {
        if let Some(api_client) = &self.api_client {
            if !pagination.is_first && pagination.token.is_none() {
                return Ok((vec![], pagination));
            }

            let mut builder = api_client
                .playlists()
                .list(&vec![
                    "id".into(),
                    "contentDetails".into(),
                    "snippet".into(),
                ])
                .mine(true)
                .max_results(50);

            if let Some(next_page) = pagination.token.clone() {
                builder = builder.page_token(next_page.as_str());
            }

            let (_, resp) = builder.doit().await?;
            let ret = if let Some(items) = resp.items {
                items.into_iter().map(|p| self.parse_playlist(p)).collect()
            } else {
                vec![]
            };

            println!("got user playlists: {:?}", ret);
            return Ok((ret, pagination.next_page_wtoken(resp.next_page_token)));
        }

        Err("API client not initialized".into())
    }

    async fn get_playlist_content(
        &self,
        playlist_id: String,
        pagination: Pagination,
    ) -> Result<(Vec<Song>, Pagination)> {
        if let Some(api_client) = &self.api_client {
            if !pagination.is_first && pagination.token.is_none() {
                return Ok((vec![], pagination));
            }

            let playlist_id = playlist_id
                .strip_prefix("youtube-playlist:")
                .unwrap_or(&playlist_id);

            let mut builder = api_client
                .playlist_items()
                .list(&vec!["id".into(), "snippet".into()])
                .playlist_id(playlist_id)
                .max_results(50);

            if let Some(next_page) = pagination.token.clone() {
                builder = builder.page_token(next_page.as_str());
            }

            let (_, resp) = builder.doit().await?;
            let ret = if let Some(items) = resp.items {
                self.fetch_song_details(
                    items
                        .iter()
                        .filter_map(|item| {
                            item.snippet.as_ref().and_then(|id| {
                                if let Some(video_id) = id.resource_id.as_ref() {
                                    video_id.video_id.clone()
                                } else {
                                    None
                                }
                            })
                        })
                        .collect(),
                )
                .await?
            } else {
                vec![]
            };

            return Ok((ret, pagination.next_page_wtoken(resp.next_page_token)));
        }

        let continuation = pagination
            .clone()
            .token
            .map(|token| serde_json::from_str::<ContinuationToken>(&token).unwrap());

        let youtube_scraper: State<YoutubeScraper> = self.app.state();
        let res = youtube_scraper
            .get_playlist_content(playlist_id, continuation)
            .await?;

        return Ok((
            res.songs,
            pagination.next_page_wtoken(
                res.next_page_token
                    .map(|token| serde_json::to_string(&token).unwrap()),
            ),
        ));
    }

    async fn get_playback_url(&self, song: Song, player: String) -> Result<String> {
        // TODO: Search spotify song
        println!("Fetching song for {} player", player);
        if player == "local" {
            let youtube_scraper: State<YoutubeScraper> = self.app.state();
            return youtube_scraper
                .get_video_url(song.song.url.clone().unwrap())
                .await;
        } else {
            return Ok(song.song.url.clone().unwrap());
        }
    }

    async fn search(&self, term: String) -> Result<SearchResult> {
        if let Some(api_client) = &self.api_client {
            let mut songs = vec![];

            let song_details = search_and_parse!(api_client, &term, "video", |item| {
                item.id.as_ref().and_then(|id| id.video_id.clone())
            });

            if !song_details.is_empty() {
                songs.extend(self.fetch_song_details(song_details).await?);
            }

            let playlists = search_and_parse!(api_client, &term, "playlist", |item| {
                item.id.as_ref().and_then(|id| {
                    id.playlist_id.as_ref().map(|playlist_id| {
                        let snippet = item.snippet.as_ref().unwrap();
                        let playlist = Playlist {
                            id: Some(playlist_id.clone()),
                            snippet: Some(PlaylistSnippet {
                                description: snippet.description.clone(),
                                thumbnails: snippet.thumbnails.clone(),
                                title: snippet.title.clone(),
                                ..Default::default()
                            }),
                            ..Default::default()
                        };
                        self.parse_playlist(playlist)
                    })
                })
            });

            let artists = search_and_parse!(api_client, &term, "channel", |item| {
                item.id.as_ref().and_then(|id| {
                    id.channel_id.as_ref().map(|channel_id| {
                        let snippet = item.snippet.as_ref().unwrap();
                        let channel = Channel {
                            id: Some(channel_id.clone()),
                            snippet: Some(ChannelSnippet {
                                description: snippet.description.clone(),
                                thumbnails: snippet.thumbnails.clone(),
                                title: snippet.title.clone(),
                                ..Default::default()
                            }),
                            ..Default::default()
                        };
                        self.parse_channel(channel)
                    })
                })
            });

            return Ok(SearchResult {
                songs,
                playlists,
                artists,
                ..Default::default()
            });
        }

        Err(MoosyncError::String("TODO".into()))

        // let youtube_scraper: State<YoutubeScraper> = self.app.state();
        // return youtube_scraper.search(term).await;
    }
}
