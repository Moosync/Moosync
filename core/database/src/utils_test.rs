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

use assertables::{assert_none, assert_ok, assert_some_eq_x};
use rstest::{fixture, rstest};
use rusqlite::Connection;
use songs_proto::moosync::types::{Album, Artist, Genre, InnerSong, Playlist, SongType};
use tracing_test::traced_test;

use crate::utils::{
    SearchByTerm, db_ms_to_proto, map_row_to_album, map_row_to_artist, map_row_to_genre,
    map_row_to_inner_song, map_row_to_playlist, proto_to_db_ms, song_type_from_str,
    song_type_to_str,
};

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn memory_connection() -> Connection {
    Connection::open_in_memory().expect("failed to open in-memory sqlite connection")
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_search_by_term_inner_song() {
    let term = Some("Rock Song".to_string());

    let song = InnerSong::search_by_term(term);
    let empty = InnerSong::search_by_term(None);

    assert_eq!(song.title.as_deref(), Some("Rock Song"));
    assert_none!(empty.title);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_search_by_term_album() {
    let term = Some("Greatest Hits".to_string());

    let album = Album::search_by_term(term);
    let empty = Album::search_by_term(None);

    assert_eq!(album.album_name.as_deref(), Some("Greatest Hits"));
    assert_none!(empty.album_name);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_search_by_term_artist() {
    let term = Some("Rock Band".to_string());

    let artist = Artist::search_by_term(term);
    let empty = Artist::search_by_term(None);

    assert_eq!(artist.artist_name.as_deref(), Some("Rock Band"));
    assert_none!(empty.artist_name);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_search_by_term_genre() {
    let term = Some("Rock".to_string());

    let genre = Genre::search_by_term(term);
    let empty = Genre::search_by_term(None);

    assert_eq!(genre.genre_name.as_deref(), Some("Rock"));
    assert_none!(empty.genre_name);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_search_by_term_playlist() {
    let term = Some("My Favorites".to_string());

    let pl = Playlist::search_by_term(term);
    let empty = Playlist::search_by_term(None);

    assert_eq!(pl.playlist_name, "My Favorites");
    assert_eq!(empty.playlist_name, "");
}

#[rstest]
#[case("LOCAL", SongType::Local as i32)]
#[case("URL", SongType::Url as i32)]
#[case("SPOTIFY", SongType::Spotify as i32)]
#[case("DASH", SongType::Dash as i32)]
#[case("HLS", SongType::Hls as i32)]
#[case("INVALID", SongType::Local as i32)]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_song_type_from_str(#[case] input: &str, #[case] expected: i32) {
    let parsed = song_type_from_str(input);

    assert_eq!(parsed, expected);
}

#[rstest]
#[case(SongType::Local as i32, "LOCAL")]
#[case(SongType::Url as i32, "URL")]
#[case(SongType::Spotify as i32, "SPOTIFY")]
#[case(SongType::Dash as i32, "DASH")]
#[case(SongType::Hls as i32, "HLS")]
#[case(999, "LOCAL")]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_song_type_to_str(#[case] input: i32, #[case] expected: &str) {
    let converted = song_type_to_str(input);

    assert_eq!(converted, expected);
}

#[rstest]
#[case(245, 750_000_000, 245750)]
#[case(0, 0, 0)]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_duration_conversions_roundtrip(
    #[case] secs: i64,
    #[case] nanos: i32,
    #[case] expected_ms: i64,
) {
    let proto_dur = Some(songs_proto::duration_proto::google::protobuf::Duration {
        seconds: secs,
        nanos,
    });

    let db_ms = proto_to_db_ms(&proto_dur);
    let converted_back = db_ms_to_proto(db_ms);

    assert_eq!(db_ms, expected_ms);
    assert_eq!(converted_back.seconds, secs);
    assert_eq!(converted_back.nanos, nanos);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_map_album_row(memory_connection: Connection) {
    memory_connection
        .execute(
            "CREATE TABLE albums (id TEXT, name TEXT, artist TEXT, cover_high TEXT, song_count INTEGER, year TEXT, cover_low TEXT)",
            [],
        )
        .unwrap();
    memory_connection
        .execute(
            "INSERT INTO albums VALUES ('alb1', 'Dark Side', 'Pink Floyd', '/cov/h.jpg', 10, '1973', '/cov/l.jpg')",
            [],
        )
        .unwrap();

    let album_res = memory_connection.query_row("SELECT * FROM albums", [], map_row_to_album);

    assert_ok!(album_res.as_ref());
    let album = album_res.unwrap();
    assert_some_eq_x!(album.album_id.as_deref(), "alb1");
    assert_some_eq_x!(album.album_name.as_deref(), "Dark Side");
    assert_some_eq_x!(album.album_artist.as_deref(), "Pink Floyd");
    assert_eq!(album.album_song_count, 10.0);
    assert_some_eq_x!(album.year.as_deref(), "1973");
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_map_artist_row(memory_connection: Connection) {
    memory_connection
        .execute(
            "CREATE TABLE artists (id TEXT, mbid TEXT, name TEXT, cover TEXT, song_count INTEGER, sanitized TEXT)",
            [],
        )
        .unwrap();
    memory_connection
        .execute(
            "INSERT INTO artists VALUES ('art1', 'mbid123', 'Queen', '/art/cov.jpg', 25, 'queen')",
            [],
        )
        .unwrap();

    let artist_res = memory_connection.query_row("SELECT * FROM artists", [], map_row_to_artist);

    assert_ok!(artist_res.as_ref());
    let artist = artist_res.unwrap();
    assert_some_eq_x!(artist.artist_id.as_deref(), "art1");
    assert_some_eq_x!(artist.artist_mbid.as_deref(), "mbid123");
    assert_some_eq_x!(artist.artist_name.as_deref(), "Queen");
    assert_eq!(artist.artist_song_count, 25.0);
    assert_some_eq_x!(artist.sanitized_artist_name.as_deref(), "queen");
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_map_genre_row(memory_connection: Connection) {
    memory_connection
        .execute(
            "CREATE TABLE genres (id TEXT, name TEXT, song_count INTEGER)",
            [],
        )
        .unwrap();
    memory_connection
        .execute("INSERT INTO genres VALUES ('gen1', 'Rock', 100)", [])
        .unwrap();

    let genre_res = memory_connection.query_row("SELECT * FROM genres", [], map_row_to_genre);

    assert_ok!(genre_res.as_ref());
    let genre = genre_res.unwrap();
    assert_some_eq_x!(genre.genre_id.as_deref(), "gen1");
    assert_some_eq_x!(genre.genre_name.as_deref(), "Rock");
    assert_eq!(genre.genre_song_count, 100.0);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_map_playlist_row(memory_connection: Connection) {
    memory_connection
        .execute(
            "CREATE TABLE playlists (id TEXT, name TEXT, cover TEXT, song_count INTEGER, desc TEXT, path TEXT, ext TEXT, icon TEXT, lib INTEGER)",
            [],
        )
        .unwrap();
    memory_connection
        .execute(
            "INSERT INTO playlists VALUES ('pl1', 'Roadtrip', '/pl/cov.jpg', 15, 'Best songs', '/path/to/pl', 'm3u', 'music-note', 1)",
            [],
        )
        .unwrap();

    let playlist_res =
        memory_connection.query_row("SELECT * FROM playlists", [], map_row_to_playlist);

    assert_ok!(playlist_res.as_ref());
    let playlist = playlist_res.unwrap();
    assert_some_eq_x!(playlist.playlist_id.as_deref(), "pl1");
    assert_eq!(playlist.playlist_name, "Roadtrip");
    assert_eq!(playlist.playlist_song_count, 15.0);
    assert_some_eq_x!(playlist.playlist_path.as_deref(), "/path/to/pl");
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_map_song_row(memory_connection: Connection) {
    memory_connection
        .execute(
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
    memory_connection
        .execute(
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

    let song_res = memory_connection.query_row("SELECT * FROM songs", [], map_row_to_inner_song);

    assert_ok!(song_res.as_ref());
    let song = song_res.unwrap();
    assert_some_eq_x!(song.id.as_deref(), "s1");
    assert_some_eq_x!(song.path.as_deref(), "/music/track.mp3");
    assert_some_eq_x!(song.title.as_deref(), "Bohemian Rhapsody");
    assert_none!(song.lyrics);
    assert_eq!(song.bitrate, Some(320.0));
    assert_eq!(song.track_no, Some(1.0));
    assert_eq!(song.r#type, SongType::Local as i32);
}
