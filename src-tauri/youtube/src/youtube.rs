use std::str::FromStr;

use rusty_ytdl::{
    search::{Channel, Playlist, SearchOptions, SearchType, YouTube},
    VideoFormat,
};

use types::{
    entities::{EntityInfo, QueryablePlaylist},
    errors::errors::{MoosyncError, Result},
};
use types::{
    entities::{QueryableAlbum, QueryableArtist, SearchResult},
    songs::{QueryableSong, Song, SongType},
};

use crate::{
    raw_parser::parse_body,
    types::{ContinuationToken, PlaylistResponse, PlaylistVideoRenderer, Root},
};

pub struct YoutubeScraper {
    youtube: YouTube,
}

impl Default for YoutubeScraper {
    fn default() -> YoutubeScraper {
        YoutubeScraper {
            youtube: YouTube::new().unwrap(),
        }
    }
}

const PLAYLIST_REGEX: &str = r"^(FL|PL|UU|LL)[a-zA-Z0-9-_]{16,41}$";
const ALBUM_REGEX: &str = r"^(RDC|O)LAK5uy_[a-zA-Z0-9-_]{33}$";
const CHANNEL_REGEX: &str = r"^UC[a-zA-Z0-9-_]{22,32}$";

impl YoutubeScraper {
    pub fn parse_song(&self, v: rusty_ytdl::search::Video) -> Song {
        Song {
            song: QueryableSong {
                _id: Some(format!("youtube:{}", v.id.clone())),
                path: None,
                size: None,
                inode: None,
                deviceno: None,
                title: Some(v.title),
                date: None,
                year: None,
                lyrics: None,
                release_type: None,
                bitrate: None,
                codec: None,
                container: None,
                duration: Some((v.duration / 1000) as f64),
                sample_rate: None,
                hash: None,
                type_: SongType::YOUTUBE,
                url: Some(v.id.clone()),
                song_cover_path_high: v.thumbnails.first().map(|d| d.url.clone()),
                song_cover_path_low: v.thumbnails.get(1).map(|d| d.url.clone()),
                playback_url: Some(v.id),
                date_added: None,
                provider_extension: None,
                icon: None,
                show_in_library: None,
                track_no: None,
            },
            album: Some(QueryableAlbum {
                album_name: Some("Misc".to_string()),
                ..Default::default()
            }),
            artists: Some(vec![QueryableArtist {
                artist_id: Some(format!("youtube-author:{}", v.channel.id)),
                artist_name: Some(v.channel.name),
                ..Default::default()
            }]),
            genre: Some(vec![]),
        }
    }

    fn parse_playlist(&self, playlist: Playlist) -> QueryablePlaylist {
        QueryablePlaylist {
            playlist_id: Some(format!("youtube-playlist:{}", playlist.id)),
            playlist_name: playlist.name,
            playlist_coverpath: playlist.thumbnails.first().map(|v| v.url.clone()),
            playlist_song_count: playlist.videos.capacity() as f64,
            ..Default::default()
        }
    }

    fn parse_artist(&self, artist: Channel) -> QueryableArtist {
        QueryableArtist {
            artist_id: Some(format!("youtube-author:{}", artist.id)),
            artist_name: Some(artist.name),
            artist_extra_info: Some(EntityInfo(
                serde_json::from_str(
                    format!(
                        r#"{{
                "youtube": {{
                    "channel_id": "{}"
                }}
            }}"#,
                        artist.id
                    )
                    .as_str(),
                )
                .unwrap(),
            )),
            artist_coverpath: artist.icon.first().map(|v| v.url.clone()),
            ..Default::default()
        }
    }

    async fn get_playlist_content_page_2(
        &self,
        continuation: ContinuationToken,
    ) -> Result<PlaylistResponse> {
        if continuation.continuation.is_none() {
            return Ok(PlaylistResponse {
                nextPageToken: None,
                songs: vec![],
            });
        }

        let mut cloned_token = continuation.clone();
        let api_key = cloned_token.api_key.clone().unwrap();
        let context = continuation.context.clone();

        let url = format!("https://www.youtube.com/youtubei/v1/browse?key={}", api_key);

        let client = reqwest::Client::new();
        cloned_token.api_key = None;
        let resp: Root = client
            .post(url)
            .json(&cloned_token)
            .send()
            .await?
            .json()
            .await?;

        if let Some(content) = resp.on_response_received_actions {
            if let Some(content) = content.first() {
                let items = content
                    .append_continuation_items_action
                    .continuation_items
                    .clone();
                let data: Vec<Song> = items
                    .into_iter()
                    .filter_map(|item| {
                        let renderer = item.playlist_video_renderer;
                        if let Some(renderer) = renderer {
                            return self.parse_video_renderer(renderer);
                        }
                        None
                    })
                    .collect();

                let continuation = content
                    .append_continuation_items_action
                    .continuation_items
                    .clone()
                    .into_iter()
                    .find(|v| v.continuation_item_renderer.is_some());

                let mut continuation_token: Option<String> = None;
                if let Some(continuation) = continuation {
                    let continuation = continuation.continuation_item_renderer.unwrap();

                    continuation_token = Some(
                        continuation
                            .continuation_endpoint
                            .continuation_command
                            .token,
                    );
                }
                return Ok(PlaylistResponse {
                    songs: data,
                    nextPageToken: Some(ContinuationToken {
                        continuation: continuation_token,
                        api_key: Some(api_key),
                        context,
                    }),
                });
            }
        }

        Err(MoosyncError::String("No data found".to_string()))
    }

    fn parse_video_renderer(&self, video_renderer: PlaylistVideoRenderer) -> Option<Song> {
        return Some(Song {
            song: QueryableSong {
                _id: Some(format!("youtube:{}", video_renderer.video_id)),
                title: Some(video_renderer.title.runs.unwrap()[0].text.clone()),

                duration: Some(
                    f64::from_str(
                        video_renderer
                            .length_seconds
                            .unwrap_or("0".to_string())
                            .as_str(),
                    )
                    .unwrap(),
                ),

                song_cover_path_high: video_renderer
                    .thumbnail
                    .thumbnails
                    .first()
                    .map(|v| v.url.clone()),
                url: Some(
                    video_renderer
                        .navigation_endpoint
                        .watch_endpoint
                        .video_id
                        .clone(),
                ),
                playback_url: Some(
                    video_renderer
                        .navigation_endpoint
                        .watch_endpoint
                        .video_id
                        .clone(),
                ),
                song_cover_path_low: video_renderer
                    .thumbnail
                    .thumbnails
                    .get(1)
                    .map(|v| v.url.clone()),
                ..Default::default()
            },
            album: None,
            artists: None,
            genre: None,
        });
    }

    fn parse_id(&self, link_or_id: String) -> Result<String> {
        if link_or_id.is_empty() {
            return Err(MoosyncError::String(
                "The linkOrId has to be a non-empty string".to_string(),
            ));
        }

        if regex::Regex::new(PLAYLIST_REGEX)
            .unwrap()
            .is_match(link_or_id.as_str())
            || regex::Regex::new(ALBUM_REGEX)
                .unwrap()
                .is_match(link_or_id.as_str())
        {
            return Ok(link_or_id.to_string());
        }

        if regex::Regex::new(CHANNEL_REGEX)
            .unwrap()
            .is_match(link_or_id.as_str())
        {
            return Ok(format!("UU{}", &link_or_id[2..]));
        }

        let parsed = url::Url::parse(link_or_id.as_str()).unwrap();

        if parsed.query_pairs().any(|(key, _)| key == "list") {
            let list_param = parsed
                .query_pairs()
                .find(|(key, _)| key == "list")
                .unwrap()
                .1;

            if regex::Regex::new(PLAYLIST_REGEX)
                .unwrap()
                .is_match(&list_param)
                || regex::Regex::new(ALBUM_REGEX)
                    .unwrap()
                    .is_match(&list_param)
            {
                return Ok(list_param.to_string());
            }

            if list_param.starts_with("RD") {
                return Err(MoosyncError::String("Mixes not supported".to_string()));
            }

            return Err(MoosyncError::String(
                "Invalid or unknown list query in url".to_string(),
            ));
        }

        let path_segments: Vec<&str> = parsed.path_segments().unwrap().collect();
        if path_segments.len() < 2 || path_segments.iter().any(|&a| a.is_empty()) {
            return Err(MoosyncError::String(format!(
                "Unable to find an id in '{}'",
                link_or_id
            )));
        }

        let maybe_type = path_segments[path_segments.len() - 2];
        let maybe_id = path_segments[path_segments.len() - 1];

        if maybe_type == "channel" && regex::Regex::new(CHANNEL_REGEX).unwrap().is_match(maybe_id) {
            return Ok(format!("UU{}", &maybe_id[2..]));
        }
        Err(MoosyncError::String("Unsupported type".to_string()))
    }

    pub async fn get_playlist_content(
        &self,
        id: String,
        continuation: Option<ContinuationToken>,
    ) -> Result<PlaylistResponse> {
        let id = self.parse_id(id)?;
        println!("fetching id {}", id);
        if let Some(continuation) = continuation {
            return self.get_playlist_content_page_2(continuation).await;
        }

        let content_url = format!("https://www.youtube.com/playlist?list={}", id);
        let content = reqwest::get(content_url).await?.text().await?;

        let (json, api_key, context) = parse_body(content.as_str());
        let parsed = json.unwrap();

        if let Some(alerts) = parsed.alerts {
            if let Some(error) = alerts
                .iter()
                .find(|a| a.alert_with_button_renderer.type_field == "ERROR")
            {
                let error_message = error.alert_with_button_renderer.text.simple_text.clone();
                return Err(MoosyncError::String(error_message));
            }
        }

        if parsed.sidebar.is_none() {
            return Err(MoosyncError::String("Unknown Playlist".into()));
        }

        let contents = parsed.contents.unwrap();
        let video_list = contents.two_column_browse_results_renderer.tabs.first();
        if let Some(video_list) = video_list {
            if let Some(video_list) = video_list.tab_renderer.content.clone() {
                if let Some(video_list) = video_list.section_list_renderer.contents.first() {
                    if let Some(video_list) = &video_list.item_section_renderer {
                        if let Some(video_list) = video_list.contents.first() {
                            let data: Vec<Song> = video_list
                                .playlist_video_list_renderer
                                .contents
                                .clone()
                                .into_iter()
                                .filter_map(|v| {
                                    let video_renderer = v.playlist_video_renderer;

                                    if let Some(video_renderer) = video_renderer {
                                        return self.parse_video_renderer(video_renderer);
                                    }
                                    None
                                })
                                .collect();

                            let mut continuation_token: Option<String> = None;
                            if let Some(continuation) = video_list
                                .playlist_video_list_renderer
                                .contents
                                .clone()
                                .into_iter()
                                .find(|v| v.continuation_item_renderer.is_some())
                            {
                                continuation_token = Some(
                                    continuation
                                        .continuation_item_renderer
                                        .unwrap()
                                        .continuation_endpoint
                                        .continuation_command
                                        .token,
                                );
                            }

                            return Ok(PlaylistResponse {
                                songs: data,
                                nextPageToken: Some(ContinuationToken {
                                    continuation: continuation_token,
                                    api_key,
                                    context,
                                }),
                            });
                        }
                    }
                }
            }
        }

        Err(MoosyncError::String("No data found".to_string()))
    }

    pub async fn search_yt(&self, title: String, artists: Vec<String>) -> Result<SearchResult> {
        let query = format!("{} - {}", artists.join(", "), title);
        let res = self
            .youtube
            .search(
                query,
                Some(&SearchOptions {
                    limit: 100,
                    search_type: SearchType::All,
                    safe_search: false,
                }),
            )
            .await?;

        let mut songs: Vec<Song> = vec![];
        let mut playlists: Vec<QueryablePlaylist> = vec![];
        let mut artists: Vec<QueryableArtist> = vec![];
        for item in res {
            match item {
                rusty_ytdl::search::SearchResult::Video(v) => songs.push(self.parse_song(v)),
                rusty_ytdl::search::SearchResult::Playlist(p) => {
                    playlists.push(self.parse_playlist(p))
                }
                rusty_ytdl::search::SearchResult::Channel(a) => artists.push(self.parse_artist(a)),
            }
        }
        Ok(SearchResult {
            songs,
            artists,
            playlists,
            albums: vec![],
            genres: vec![],
        })
    }

    pub async fn get_video_url(&self, id: String) -> Result<String> {
        let video = rusty_ytdl::Video::new(id)?;
        let info = video.get_info().await?;

        let mut best_format: Option<VideoFormat> = None;
        for format in info.formats {
            if let Some(best) = best_format.clone() {
                if format.has_audio
                    && !format.is_dash_mpd
                    && format.is_hls
                    && format.audio_bitrate > best.audio_bitrate
                {
                    best_format = Some(format);
                }
            } else {
                best_format = Some(format);
            }
        }

        match best_format {
            Some(f) => Ok(f.url.clone()),
            None => Err(MoosyncError::String("Unable to find URL".into())),
        }
    }
}
