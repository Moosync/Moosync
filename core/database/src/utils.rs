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

use songs_proto::moosync::types::{Album, Artist, Genre, InnerSong, Playlist, SongType};

pub trait SearchByTerm {
    fn search_by_term(term: Option<String>) -> Self;
}

impl SearchByTerm for InnerSong {
    #[tracing::instrument(level = "debug", skip_all)]
    fn search_by_term(term: Option<String>) -> Self {
        let mut data = Self::default();
        data.title.clone_from(&term);
        data.path = term;
        data
    }
}

impl SearchByTerm for Album {
    #[tracing::instrument(level = "debug", skip_all)]
    fn search_by_term(term: Option<String>) -> Self {
        Self {
            album_name: term,
            ..Default::default()
        }
    }
}

impl SearchByTerm for Artist {
    #[tracing::instrument(level = "debug", skip_all)]
    fn search_by_term(term: Option<String>) -> Self {
        Self {
            artist_name: term,
            ..Default::default()
        }
    }
}

impl SearchByTerm for Genre {
    #[tracing::instrument(level = "debug", skip_all)]
    fn search_by_term(term: Option<String>) -> Self {
        Self {
            genre_name: term,
            ..Default::default()
        }
    }
}

impl SearchByTerm for Playlist {
    #[tracing::instrument(level = "debug", skip_all)]
    fn search_by_term(term: Option<String>) -> Self {
        Self {
            playlist_name: term.unwrap_or_default(),
            ..Default::default()
        }
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn song_type_from_str(s: &str) -> i32 {
    let t = match s {
        "LOCAL" => SongType::Local,
        "URL" => SongType::Url,
        "SPOTIFY" => SongType::Spotify,
        "DASH" => SongType::Dash,
        "HLS" => SongType::Hls,
        _ => SongType::Local,
    };
    t as i32
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn song_type_to_str(t: i32) -> &'static str {
    match SongType::try_from(t).unwrap_or(SongType::Local) {
        SongType::Local => "LOCAL",
        SongType::Url => "URL",
        SongType::Spotify => "SPOTIFY",
        SongType::Dash => "DASH",
        SongType::Hls => "HLS",
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn map_row_to_album(row: &rusqlite::Row) -> rusqlite::Result<Album> {
    Ok(Album {
        album_id: row.get(0)?,
        album_name: row.get(1)?,
        album_artist: row.get(2)?,
        album_coverpath_high: row.get(3)?,
        album_song_count: row.get(4)?,
        year: row.get(5)?,
        album_coverpath_low: row.get(6)?,
    })
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn map_row_to_artist(row: &rusqlite::Row) -> rusqlite::Result<Artist> {
    Ok(Artist {
        artist_id: row.get(0)?,
        artist_mbid: row.get(1)?,
        artist_name: row.get(2)?,
        artist_coverpath: row.get(3)?,
        artist_song_count: row.get(4)?,
        sanitized_artist_name: row.get(5)?,
    })
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn map_row_to_genre(row: &rusqlite::Row) -> rusqlite::Result<Genre> {
    Ok(Genre {
        genre_id: row.get(0)?,
        genre_name: row.get(1)?,
        genre_song_count: row.get(2)?,
    })
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn map_row_to_playlist(row: &rusqlite::Row) -> rusqlite::Result<Playlist> {
    Ok(Playlist {
        playlist_id: row.get(0)?,
        playlist_name: row.get(1).unwrap_or_default(),
        playlist_coverpath: row.get(2)?,
        playlist_song_count: row.get(3).unwrap_or_default(),
        playlist_desc: row.get(4)?,
        playlist_path: row.get(5)?,
        extension: row.get(6)?,
        icon: row.get(7)?,
        library_item: row.get(8)?,
    })
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn map_row_to_inner_song(row: &rusqlite::Row) -> rusqlite::Result<InnerSong> {
    let type_str: String = row.get(16)?;
    Ok(InnerSong {
        id: row.get(0)?,
        path: row.get(1)?,
        size: row.get(2)?,
        inode: row.get(3)?,
        deviceno: row.get(4)?,
        title: row.get(5)?,
        date: row.get(6)?,
        year: row.get(7)?,
        lyrics: row.get(8)?,
        release_type: row.get(9)?,
        bitrate: row.get(10)?,
        codec: row.get(11)?,
        container: row.get(12)?,
        duration: Some(db_ms_to_proto(row.get::<_, f64>(13)? as i64)),
        sample_rate: row.get(14)?,
        hash: row.get(15)?,
        r#type: song_type_from_str(&type_str),
        url: row.get(17)?,
        song_cover_path_high: row.get(18)?,
        playback_url: row.get(19)?,
        song_cover_path_low: row.get(20)?,
        date_added: row.get(21)?,
        provider_extension: row.get(22)?,
        icon: row.get(23)?,
        show_in_library: row.get(24)?,
        track_no: row.get(25)?,
        library_item: row.get(26)?,
    })
}

pub(crate) fn proto_to_db_ms(
    proto_dur: &Option<songs_proto::duration_proto::google::protobuf::Duration>,
) -> i64 {
    match proto_dur.as_ref() {
        Some(dur) => (dur.seconds * 1000) + (dur.nanos as i64 / 1_000_000),
        None => 0,
    }
}

// Convert DB milliseconds back to Protobuf Duration
pub(crate) fn db_ms_to_proto(
    db_ms: i64,
) -> songs_proto::duration_proto::google::protobuf::Duration {
    songs_proto::duration_proto::google::protobuf::Duration {
        seconds: db_ms / 1000,
        // Remainder converted back to nanos
        nanos: ((db_ms % 1000) * 1_000_000) as i32,
    }
}
