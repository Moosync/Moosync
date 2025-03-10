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

use std::str::FromStr;

use rusty_ytdl::{
    reqwest::Url,
    search::{Channel, Playlist, SearchOptions, SearchType, YouTube},
};

use types::{
    entities::{EntityInfo, QueryablePlaylist},
    errors::{MoosyncError, Result},
    providers::generic::Pagination,
};
use types::{
    entities::{QueryableAlbum, QueryableArtist, SearchResult},
    songs::{QueryableSong, Song, SongType},
};

use crate::types::PlaylistResponse;

pub struct YoutubeScraper {
    youtube: YouTube,
}

impl Default for YoutubeScraper {
    #[tracing::instrument(level = "debug", skip())]
    fn default() -> YoutubeScraper {
        YoutubeScraper {
            youtube: YouTube::new().unwrap(),
        }
    }
}

impl YoutubeScraper {
    #[tracing::instrument(level = "debug", skip(self, v))]
    fn parse_song(&self, v: &rusty_ytdl::search::Video) -> Song {
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
                artist_id: Some(format!("youtube-artist:{}", v.channel.id)),
                artist_name: Some(v.channel.name.clone()),
                ..Default::default()
            }]),
            genre: Some(vec![]),
        }
    }

    #[tracing::instrument(level = "debug", skip(self, v))]
    fn parse_video_info(&self, v: &rusty_ytdl::VideoInfo) -> Song {
        let details = &v.video_details;
        Song {
            song: QueryableSong {
                _id: Some(format!("youtube:{}", details.video_id.clone())),
                deviceno: None,
                title: Some(details.title.clone()),
                duration: Some(details.length_seconds.parse().unwrap_or_default()),
                type_: SongType::YOUTUBE,
                url: Some(details.video_id.clone()),
                song_cover_path_high: details.thumbnails.first().map(|d| d.url.clone()),
                song_cover_path_low: details.thumbnails.get(1).map(|d| d.url.clone()),
                playback_url: Some(details.video_id.clone()),
                provider_extension: Some("youtube".into()),
                ..Default::default()
            },
            album: Some(QueryableAlbum {
                album_name: Some("Misc".to_string()),
                ..Default::default()
            }),
            artists: Some(vec![QueryableArtist {
                artist_id: Some(format!("youtube-artist:{}", details.channel_id)),
                artist_name: Some(details.owner_channel_name.clone()),
                ..Default::default()
            }]),
            genre: Some(vec![]),
        }
    }

    #[tracing::instrument(level = "debug", skip(self, playlist))]
    fn parse_playlist(&self, playlist: &Playlist) -> QueryablePlaylist {
        QueryablePlaylist {
            playlist_id: Some(format!("youtube-playlist:{}", playlist.id)),
            playlist_name: playlist.name.clone(),
            playlist_coverpath: playlist.thumbnails.first().map(|v| v.url.clone()),
            playlist_song_count: playlist.videos.capacity() as f64,
            extension: Some("youtube".into()),
            ..Default::default()
        }
    }

    #[tracing::instrument(level = "debug", skip(self, artist))]
    fn parse_artist(&self, artist: &Channel) -> QueryableArtist {
        QueryableArtist {
            artist_id: Some(format!("youtube-artist:{}", artist.id)),
            artist_name: Some(artist.name.clone()),
            artist_extra_info: Some(EntityInfo(format!(
                r#"{{
                "youtube": {{
                    "channel_id": "{}"
                }}
            }}"#,
                artist.id
            ))),
            artist_coverpath: artist.icon.first().map(|v| v.url.clone()),
            ..Default::default()
        }
    }

    #[tracing::instrument(level = "debug", skip(self, id))]
    pub async fn get_playlist_content(
        &self,
        id: String,
        _: Pagination,
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

    #[tracing::instrument(level = "debug", skip(self, id))]
    pub async fn get_video_by_id(&self, id: String) -> Result<Song> {
        let video = rusty_ytdl::Video::new(id)?;
        let info = video.get_basic_info().await?;
        Ok(self.parse_video_info(&info))
    }

    #[tracing::instrument(level = "debug", skip(self, query))]
    pub async fn search_yt(&self, query: impl Into<String>) -> Result<SearchResult> {
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
                rusty_ytdl::search::SearchResult::Video(v) => songs.push(self.parse_song(&v)),
                rusty_ytdl::search::SearchResult::Playlist(p) => {
                    playlists.push(self.parse_playlist(&p))
                }
                rusty_ytdl::search::SearchResult::Channel(a) => artists.push(self.parse_artist(&a)),
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

    #[tracing::instrument(level = "debug", skip(self, id))]
    pub async fn get_video_url(&self, mut id: String) -> Result<String> {
        if id.starts_with("http") {
            let url = Url::from_str(&id).unwrap();
            let query = url.query_pairs().find(|(k, v)| k == "v");
            if let Some((_, v)) = query {
                id = v.to_string();
            }
        }
        let video = rusty_ytdl::Video::new(id)?;
        let info = video.get_info().await?;

        tracing::debug!("Got formats {:?}", info.formats);

        let best_format = info
            .formats
            .into_iter()
            .filter(|format| format.has_audio && !format.has_video)
            .max_by(|a, b| a.bitrate.cmp(&b.bitrate));

        tracing::debug!("chose formats {:?}", best_format);

        match best_format {
            Some(f) => Ok(f.url.clone()),
            None => Err(MoosyncError::String("Unable to find URL".into())),
        }
    }

    pub async fn get_suggestions(&self) -> Result<Vec<Song>> {
        let songs = self
            .youtube
            .search(
                "music video",
                Some(&SearchOptions {
                    limit: 100,
                    search_type: SearchType::Video,
                    safe_search: false,
                }),
            )
            .await?;

        Ok(songs
            .into_iter()
            .filter_map(|s| {
                if let rusty_ytdl::search::SearchResult::Video(v) = s {
                    Some(self.parse_song(&v))
                } else {
                    None
                }
            })
            .collect())
    }
}
