use database::types::{
    entities::{QueryableAlbum, QueryableArtist, SearchResult},
    songs::{QueryableSong, Song, SongType},
};
use rusty_ytdl::{
    search::{SearchOptions, SearchType, YouTube},
    VideoFormat,
};
use types::errors::errors::{MoosyncError, Result};

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
        println!("getting video url for {}", id);
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
