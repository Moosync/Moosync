use assertables::{assert_none, assert_ok, assert_some_eq_x};
use rstest::rstest;
use songs_proto::moosync::types::{Album, Artist, Genre, InnerSong, Playlist, SongType};
use sqlx::SqlitePool;
use tracing_test::traced_test;

use crate::{
    models::{DbAlbumRow, DbArtistRow, DbGenreRow, DbInnerSongRow, DbPlaylistRow},
    utils::{SearchByTerm, db_ms_to_proto, proto_to_db_ms, song_type_from_str, song_type_to_str},
};

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

#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_map_album_row() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::raw_sql(
        "CREATE TABLE albums (album_id TEXT, album_name TEXT, album_artist TEXT, album_coverpath_high TEXT, album_song_count INTEGER, year TEXT, album_coverpath_low TEXT);
         INSERT INTO albums VALUES ('alb1', 'Dark Side', 'Pink Floyd', '/cov/h.jpg', 10, '1973', '/cov/l.jpg');",
    )
    .execute(&pool)
    .await
    .unwrap();

    let album_res: Result<DbAlbumRow, _> = sqlx::query_as("SELECT * FROM albums")
        .fetch_one(&pool)
        .await;

    assert_ok!(album_res.as_ref());
    let album: Album = album_res.unwrap().into();
    assert_some_eq_x!(album.album_id.as_deref(), "alb1");
    assert_some_eq_x!(album.album_name.as_deref(), "Dark Side");
    assert_some_eq_x!(album.album_artist.as_deref(), "Pink Floyd");
    assert_eq!(album.album_song_count, 10.0);
    assert_some_eq_x!(album.year.as_deref(), "1973");
}

#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_map_artist_row() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::raw_sql(
        "CREATE TABLE artists (artist_id TEXT, artist_mbid TEXT, artist_name TEXT, artist_coverpath TEXT, artist_song_count INTEGER, sanitized_artist_name TEXT);
         INSERT INTO artists VALUES ('art1', 'mbid123', 'Queen', '/art/cov.jpg', 25, 'queen');",
    )
    .execute(&pool)
    .await
    .unwrap();

    let artist_res: Result<DbArtistRow, _> = sqlx::query_as("SELECT * FROM artists")
        .fetch_one(&pool)
        .await;

    assert_ok!(artist_res.as_ref());
    let artist: Artist = artist_res.unwrap().into();
    assert_some_eq_x!(artist.artist_id.as_deref(), "art1");
    assert_some_eq_x!(artist.artist_mbid.as_deref(), "mbid123");
    assert_some_eq_x!(artist.artist_name.as_deref(), "Queen");
    assert_eq!(artist.artist_song_count, 25.0);
    assert_some_eq_x!(artist.sanitized_artist_name.as_deref(), "queen");
}

#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_map_genre_row() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::raw_sql(
        "CREATE TABLE genres (genre_id TEXT, genre_name TEXT, genre_song_count INTEGER);
         INSERT INTO genres VALUES ('gen1', 'Rock', 100);",
    )
    .execute(&pool)
    .await
    .unwrap();

    let genre_res: Result<DbGenreRow, _> = sqlx::query_as("SELECT * FROM genres")
        .fetch_one(&pool)
        .await;

    assert_ok!(genre_res.as_ref());
    let genre: Genre = genre_res.unwrap().into();
    assert_some_eq_x!(genre.genre_id.as_deref(), "gen1");
    assert_some_eq_x!(genre.genre_name.as_deref(), "Rock");
    assert_eq!(genre.genre_song_count, 100.0);
}

#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_map_playlist_row() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::raw_sql(
        "CREATE TABLE playlists (playlist_id TEXT, playlist_name TEXT, playlist_coverpath TEXT, playlist_song_count INTEGER, playlist_desc TEXT, playlist_path TEXT, extension TEXT, icon TEXT, library_item INTEGER);
         INSERT INTO playlists VALUES ('pl1', 'Roadtrip', '/pl/cov.jpg', 15, 'Best songs', '/path/to/pl', 'm3u', 'music-note', 1);",
    )
    .execute(&pool)
    .await
    .unwrap();

    let playlist_res: Result<DbPlaylistRow, _> = sqlx::query_as("SELECT * FROM playlists")
        .fetch_one(&pool)
        .await;

    assert_ok!(playlist_res.as_ref());
    let playlist: Playlist = playlist_res.unwrap().into();
    assert_some_eq_x!(playlist.playlist_id.as_deref(), "pl1");
    assert_eq!(playlist.playlist_name, "Roadtrip");
    assert_eq!(playlist.playlist_song_count, 15.0);
    assert_some_eq_x!(playlist.playlist_path.as_deref(), "/path/to/pl");
}

#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_map_song_row() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::raw_sql(
        "CREATE TABLE songs (
            id TEXT, path TEXT, size REAL, title TEXT, date TEXT, year TEXT,
            release_type TEXT, bitrate REAL, codec TEXT, container TEXT, duration INTEGER,
            sample_rate REAL, hash TEXT, type TEXT, url TEXT, song_cover_path_high TEXT,
            playback_url TEXT, song_cover_path_low TEXT, date_added INTEGER, extension TEXT,
            track_no REAL
        );
        INSERT INTO songs VALUES (
            's1', '/music/track.mp3', 1024.0, 'Bohemian Rhapsody', '1975', '1975',
            'album', 320.0, 'mp3', 'id3', 354000, 44100.0, 'hash123', 'LOCAL',
            'https://moosync.app', '/cov/h.jpg', 'https://stream.mp3', '/cov/l.jpg',
            1700000000, '', 1.0
        );",
    )
    .execute(&pool)
    .await
    .unwrap();

    let song_res: Result<DbInnerSongRow, _> =
        sqlx::query_as("SELECT * FROM songs").fetch_one(&pool).await;

    assert_ok!(song_res.as_ref());
    let song: InnerSong = song_res.unwrap().into();
    assert_some_eq_x!(song.id.as_deref(), "s1");
    assert_some_eq_x!(song.path.as_deref(), "/music/track.mp3");
    assert_some_eq_x!(song.title.as_deref(), "Bohemian Rhapsody");
    assert_none!(song.lyrics);
    assert_eq!(song.bitrate, Some(320.0));
    assert_eq!(song.track_no, Some(1.0));
    assert_eq!(song.r#type, SongType::Local as i32);
}
