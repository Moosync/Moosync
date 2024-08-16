use std::str::FromStr;

use rusty_ytdl::{
    search::{Channel, Playlist, SearchOptions, SearchType, YouTube},
    VideoFormat,
};

use types::{
    entities::{EntityInfo, QueryablePlaylist},
    errors::errors::{MoosyncError, Result},
    providers::generic::Pagination,
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
    pub fn parse_song(&self, v: &rusty_ytdl::search::Video) -> Song {
        Song {
            song: QueryableSong {
                _id: Some(format!("youtube:{}", v.id.clone())),
                deviceno: None,
                title: Some(v.title.clone()),
                duration: Some((v.duration / 1000) as f64),
                type_: SongType::YOUTUBE,
                url: Some(v.id.clone()),
                song_cover_path_high: v.thumbnails.first().map(|d| d.url.clone()),
                song_cover_path_low: v.thumbnails.get(1).map(|d| d.url.clone()),
                playback_url: Some(v.id.clone()),
                provider_extension: Some("youtube".into()),
                ..Default::default()
            },
            album: Some(QueryableAlbum {
                album_name: Some("Misc".to_string()),
                ..Default::default()
            }),
            artists: Some(vec![QueryableArtist {
                artist_id: Some(format!("youtube-author:{}", v.channel.id)),
                artist_name: Some(v.channel.name.clone()),
                ..Default::default()
            }]),
            genre: Some(vec![]),
        }
    }

    fn parse_playlist(&self, playlist: &Playlist) -> QueryablePlaylist {
        QueryablePlaylist {
            playlist_id: Some(format!("youtube-playlist:{}", playlist.id)),
            playlist_name: playlist.name.clone(),
            playlist_coverpath: playlist.thumbnails.first().map(|v| v.url.clone()),
            playlist_song_count: playlist.videos.capacity() as f64,
            ..Default::default()
        }
    }

    fn parse_artist(&self, artist: &Channel) -> QueryableArtist {
        QueryableArtist {
            artist_id: Some(format!("youtube-author:{}", artist.id)),
            artist_name: Some(artist.name.clone()),
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

    pub async fn get_playlist_content(
        &self,
        id: String,
        pagination: Pagination,
    ) -> Result<PlaylistResponse> {
        let mut playlist = rusty_ytdl::search::Playlist::get(id, None).await?;
        playlist.fetch(None).await;
        let res = playlist.videos.iter().map(|v| self.parse_song(v)).collect();

        Ok(PlaylistResponse {
            songs: res,
            next_page_token: None,
        })

        // Err(MoosyncError::String("No data found".to_string()))
    }

    pub async fn search_yt(&self, query: String) -> Result<SearchResult> {
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
        for item in res.iter() {
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
