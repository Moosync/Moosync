use std::fs;

use database::{
    cache::CacheHolder,
    database::Database,
    types::{
        entities::{GetEntityOptions, SearchResult},
        songs::{GetSongOptions, QueryableSong, Song},
    },
};
use macros::generate_command;
use serde_json::Value;
use tauri::{App, Manager, State};

generate_command!(insert_songs, Database, Vec<Song>, songs: Vec<Song>);
generate_command!(remove_songs, Database, (), songs: Vec<String>);
generate_command!(update_song, Database, (), a: QueryableSong);
generate_command!(get_songs_by_options, Database, Vec<Song>, options: GetSongOptions);
generate_command!(get_entity_by_options, Database, Value, options: GetEntityOptions);
generate_command!(search_all, Database, SearchResult, term: String);

pub fn get_cache_state(app: &mut App) -> CacheHolder {
    let path = app.path().app_cache_dir().unwrap().join("http_cache.db");
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).unwrap();
        }
    }
    println!("Cache DB path {:?}", path);
    let db = CacheHolder::new(path);

    db
}

pub fn get_db_state(app: &mut App) -> Database {
    let path = app.path().app_data_dir().unwrap().join("songs.db");
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).unwrap();
        }
    }
    println!("DB path {:?}", path);

    Database::new(path)
}
