use std::fs;

use database::{cache::CacheHolder, database::Database};
use macros::generate_command;
use serde_json::Value;
use tauri::{App, Manager, State};
use types::{
    entities::{
        GetEntityOptions, QueryableAlbum, QueryableArtist, QueryablePlaylist, SearchResult,
    },
    songs::{GetSongOptions, QueryableSong, Song},
};

generate_command!(insert_songs, Database, Vec<Song>, songs: Vec<Song>);
generate_command!(remove_songs, Database, (), songs: Vec<String>);
generate_command!(update_song, Database, (), a: QueryableSong);
generate_command!(get_songs_by_options, Database, Vec<Song>, options: GetSongOptions);
generate_command!(get_entity_by_options, Database, Value, options: GetEntityOptions);
generate_command!(search_all, Database, SearchResult, term: String);
generate_command!(create_playlist, Database, String, playlist: QueryablePlaylist);
generate_command!(add_to_playlist, Database, (), id: String, songs: Vec<Song>);
generate_command!(remove_from_playlist, Database, (), id: String, songs: Vec<String>);
generate_command!(remove_playlist, Database, (), id: String);
generate_command!(update_album, Database, (), album: QueryableAlbum);
generate_command!(update_artist, Database, (), artist: QueryableArtist);
generate_command!(update_playlist, Database, (), playlist: QueryablePlaylist);
generate_command!(update_songs, Database, (), songs: Vec<Song>);
generate_command!(update_lyrics, Database, (), id: String, lyrics: String);
generate_command!(increment_play_count, Database, (), id: String);
generate_command!(increment_play_time, Database, (), id: String, duration: f64);

pub fn get_cache_state(app: &mut App) -> CacheHolder {
    let path = app.path().app_cache_dir().unwrap().join("http_cache.db");
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).unwrap();
        }
    }

    CacheHolder::new(path)
}

pub fn get_db_state(app: &mut App) -> Database {
    let path = app.path().app_data_dir().unwrap().join("songs.db");
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).unwrap();
        }
    }

    Database::new(path)
}
