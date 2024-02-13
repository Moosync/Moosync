use crate::{
    errors::errors::Result,
    generate_command_async,
    types::{
        entities::{QueryableAlbum, QueryableArtist, SearchResult},
        songs::{QueryableSong, Song, SongType},
    },
};

use rusty_ytdl::{
    search::{SearchOptions, SearchType, YouTube},
    Video,
};

use tauri::State;
pub struct YoutubeScraper {
    youtube: YouTube,
}

impl YoutubeScraper {
    pub fn new() -> YoutubeScraper {
        YoutubeScraper {
            youtube: YouTube::new().unwrap(),
        }
    }

    fn parse_song(&self, v: rusty_ytdl::search::Video) -> Song {
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
                song_cover_path_high: v.thumbnails.get(0).map(|d| d.url.clone()),
                song_cover_path_low: v.thumbnails.get(1).map(|d| d.url.clone()),
                playback_url: Some(v.id),
                date_added: None,
                provider_extension: None,
                icon: None,
                show_in_library: None,
                track_no: None,
            },
            album: Some(QueryableAlbum {
                album_id: None,
                album_name: Some("Misc".to_string()),
                album_artist: None,
                album_coverPath_high: None,
                album_song_count: 00f64,
                year: None,
                album_coverPath_low: None,
                album_extra_info: None,
            }),
            artists: vec![QueryableArtist {
                artist_id: Some(format!("youtube-author:{}", v.channel.id)),
                artist_mbid: None,
                artist_name: Some(v.channel.name),
                artist_coverPath: None,
                artist_song_count: 0f64,
                artist_extra_info: None,
                sanitized_artist_name: None,
            }],
            genre: vec![],
        }
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
        for item in res {
            match item {
                rusty_ytdl::search::SearchResult::Video(v) => songs.push(self.parse_song(v)),
                rusty_ytdl::search::SearchResult::Playlist(_) => {}
                rusty_ytdl::search::SearchResult::Channel(_) => {}
            }
        }
        Ok(SearchResult {
            songs,
            artists: vec![],
            playlists: vec![],
            albums: vec![],
            genres: vec![],
        })
    }

    pub async fn get_video_url(&self, id: String) -> Result<String> {
        let video = rusty_ytdl::Video::new(id)?;
        Ok(video.get_video_url())
    }
}

pub fn get_youtube_scraper_state() -> YoutubeScraper {
    YoutubeScraper::new()
}

generate_command_async!(search_yt, YoutubeScraper, SearchResult, title: String, artists: Vec<String>);
generate_command_async!(get_video_url, YoutubeScraper, String, id: String);
