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

use rusqlite::Connection;
use songs_proto::moosync::types::{Album, Artist, Genre, InnerSong, Playlist, SongType};

use crate::utils::{
    SearchByTerm, db_ms_to_proto, map_row_to_album, map_row_to_artist, map_row_to_genre,
    map_row_to_inner_song, map_row_to_playlist, proto_to_db_ms, song_type_from_str,
    song_type_to_str,
};

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_search_by_term_inner_song() {
    let term = Some("Rock Song".to_string());
    let song = InnerSong::search_by_term(term.clone());
    assert_eq!(song.title, term);
    assert_eq!(song.path, term);

    let empty_song = InnerSong::search_by_term(None);
    assert!(empty_song.title.is_none());
    assert!(empty_song.path.is_none());
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_search_by_term_album() {
    let term = Some("Abbey Road".to_string());
    let album = Album::search_by_term(term.clone());
    assert_eq!(album.album_name, term);

    let empty = Album::search_by_term(None);
    assert!(empty.album_name.is_none());
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_search_by_term_artist() {
    let term = Some("Queen".to_string());
    let artist = Artist::search_by_term(term.clone());
    assert_eq!(artist.artist_name, term);

    let empty = Artist::search_by_term(None);
    assert!(empty.artist_name.is_none());
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_search_by_term_genre() {
    let term = Some("Electronic".to_string());
    let genre = Genre::search_by_term(term.clone());
    assert_eq!(genre.genre_name, term);

    let empty = Genre::search_by_term(None);
    assert!(empty.genre_name.is_none());
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_search_by_term_playlist() {
    let term = Some("My Favorites".to_string());
    let pl = Playlist::search_by_term(term);
    assert_eq!(pl.playlist_name, "My Favorites");

    let empty = Playlist::search_by_term(None);
    assert_eq!(empty.playlist_name, "");
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_song_type_conversions_all_variants() {
    assert_eq!(song_type_from_str("LOCAL"), SongType::Local as i32);
    assert_eq!(song_type_from_str("URL"), SongType::Url as i32);
    assert_eq!(song_type_from_str("SPOTIFY"), SongType::Spotify as i32);
    assert_eq!(song_type_from_str("DASH"), SongType::Dash as i32);
    assert_eq!(song_type_from_str("HLS"), SongType::Hls as i32);
    assert_eq!(song_type_from_str("INVALID"), SongType::Local as i32);

    assert_eq!(song_type_to_str(SongType::Local as i32), "LOCAL");
    assert_eq!(song_type_to_str(SongType::Url as i32), "URL");
    assert_eq!(song_type_to_str(SongType::Spotify as i32), "SPOTIFY");
    assert_eq!(song_type_to_str(SongType::Dash as i32), "DASH");
    assert_eq!(song_type_to_str(SongType::Hls as i32), "HLS");
    assert_eq!(song_type_to_str(999), "LOCAL");
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_duration_conversions_roundtrip() {
    let proto_dur = Some(songs_proto::duration_proto::google::protobuf::Duration {
        seconds: 245,
        nanos: 750_000_000,
    });
    let db_ms = proto_to_db_ms(&proto_dur);
    assert_eq!(db_ms, 245750);

    let converted_back = db_ms_to_proto(db_ms);
    assert_eq!(converted_back.seconds, 245);
    assert_eq!(converted_back.nanos, 750_000_000);

    let zero_ms = proto_to_db_ms(&None);
    assert_eq!(zero_ms, 0);

    let zero_proto = db_ms_to_proto(0);
    assert_eq!(zero_proto.seconds, 0);
    assert_eq!(zero_proto.nanos, 0);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_map_album_row() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute(
        "CREATE TABLE albums (id TEXT, name TEXT, artist TEXT, cover_high TEXT, song_count INTEGER, year TEXT, cover_low TEXT)",
        [],
    ).unwrap();
    conn.execute(
        "INSERT INTO albums VALUES ('alb1', 'Dark Side', 'Pink Floyd', '/cov/h.jpg', 10, '1973', '/cov/l.jpg')",
        [],
    ).unwrap();
    let album = conn
        .query_row("SELECT * FROM albums", [], |row| map_row_to_album(row))
        .unwrap();
    assert_eq!(album.album_id, Some("alb1".to_string()));
    assert_eq!(album.album_name, Some("Dark Side".to_string()));
    assert_eq!(album.album_artist, Some("Pink Floyd".to_string()));
    assert_eq!(album.album_song_count, 10.0);
    assert_eq!(album.year, Some("1973".to_string()));
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_map_artist_row() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute(
        "CREATE TABLE artists (id TEXT, mbid TEXT, name TEXT, cover TEXT, song_count INTEGER, sanitized TEXT)",
        [],
    ).unwrap();
    conn.execute(
        "INSERT INTO artists VALUES ('art1', 'mbid123', 'Queen', '/art/cov.jpg', 25, 'queen')",
        [],
    )
    .unwrap();
    let artist = conn
        .query_row("SELECT * FROM artists", [], |row| map_row_to_artist(row))
        .unwrap();
    assert_eq!(artist.artist_id, Some("art1".to_string()));
    assert_eq!(artist.artist_mbid, Some("mbid123".to_string()));
    assert_eq!(artist.artist_name, Some("Queen".to_string()));
    assert_eq!(artist.artist_song_count, 25.0);
    assert_eq!(artist.sanitized_artist_name, Some("queen".to_string()));
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_map_genre_row() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute(
        "CREATE TABLE genres (id TEXT, name TEXT, song_count INTEGER)",
        [],
    )
    .unwrap();
    conn.execute("INSERT INTO genres VALUES ('gen1', 'Rock', 100)", [])
        .unwrap();
    let genre = conn
        .query_row("SELECT * FROM genres", [], |row| map_row_to_genre(row))
        .unwrap();
    assert_eq!(genre.genre_id, Some("gen1".to_string()));
    assert_eq!(genre.genre_name, Some("Rock".to_string()));
    assert_eq!(genre.genre_song_count, 100.0);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_map_playlist_row() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute(
        "CREATE TABLE playlists (id TEXT, name TEXT, cover TEXT, song_count INTEGER, desc TEXT, path TEXT, ext TEXT, icon TEXT, lib INTEGER)",
        [],
    ).unwrap();
    conn.execute(
        "INSERT INTO playlists VALUES ('pl1', 'Roadtrip', '/pl/cov.jpg', 15, 'Best songs', '/path/to/pl', 'm3u', 'music-note', 1)",
        [],
    ).unwrap();
    let playlist = conn
        .query_row("SELECT * FROM playlists", [], |row| {
            map_row_to_playlist(row)
        })
        .unwrap();
    assert_eq!(playlist.playlist_id, Some("pl1".to_string()));
    assert_eq!(playlist.playlist_name, "Roadtrip");
    assert_eq!(playlist.playlist_song_count, 15.0);
    assert_eq!(playlist.playlist_path, Some("/path/to/pl".to_string()));
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_map_song_row() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute(
        "CREATE TABLE songs (
            id TEXT, path TEXT, size REAL, dummy1 TEXT, dummy2 TEXT,
            title TEXT, date TEXT, year TEXT, lyrics TEXT, release_type TEXT,
            bitrate REAL, codec TEXT, container TEXT, duration REAL, sample_rate REAL,
            hash TEXT, type TEXT, url TEXT, cover_high TEXT, playback_url TEXT,
            cover_low TEXT, date_added INTEGER, d3 TEXT, d4 TEXT, d5 TEXT, track_no REAL
        )",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO songs VALUES (
            's1', '/music/track.mp3', 1024.0, '', '',
            'Bohemian Rhapsody', '1975', '1975', 'Mama, just killed a man...', 'album',
            320.0, 'mp3', 'id3', 354000.0, 44100.0,
            'hash123', 'LOCAL', 'https://moosync.app', '/cov/h.jpg', 'https://stream.mp3',
            '/cov/l.jpg', 1700000000, '', '', '', 1
        )",
        [],
    )
    .unwrap();
    let song = conn
        .query_row("SELECT * FROM songs", [], |row| map_row_to_inner_song(row))
        .unwrap();
    assert_eq!(song.id, Some("s1".to_string()));
    assert_eq!(song.path, Some("/music/track.mp3".to_string()));
    assert_eq!(song.title, Some("Bohemian Rhapsody".to_string()));
    assert_eq!(song.bitrate, Some(320.0));
    assert_eq!(song.track_no, Some(1.0));
    assert_eq!(song.r#type, SongType::Local as i32);
}
