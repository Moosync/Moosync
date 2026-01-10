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

use crate::macros::generate_command;
use database::{cache::CacheHolder, database::Database};
use serde_json::Value;
use songs_proto::moosync::types::AllAnalytics;
use songs_proto::moosync::types::{
    Album, Artist, GetEntityOptions, GetSongOptions, InnerSong, Playlist, SearchResult, Song,
};
use std::fs;
use tauri::{App, AppHandle, Manager, State};
use tracing::{info, trace};
use types::errors::{Result, error_helpers};

use crate::window::handler::WindowHandler;

#[tracing::instrument(level = "debug", skip(app, db, window_handler))]
#[tauri_invoke_proc::parse_tauri_command]
#[tauri::command(async)]
pub fn export_playlist(
    app: AppHandle,
    db: State<Database>,
    window_handler: State<WindowHandler>,
    id: String,
) -> Result<()> {
    info!("Exporting playlist");
    let exported = db.export_playlist(id)?;
    let selected_file = window_handler.open_save_file(app)?;
    trace!("Exported playlist");
    fs::write(selected_file, exported).map_err(error_helpers::to_file_system_error)?;

    Ok(())
}

generate_command!(insert_songs, Database, Vec<Song>, songs: Vec<Song>);
generate_command!(remove_songs, Database, (), songs: Vec<String>);
generate_command!(update_song, Database, (), a: InnerSong);
generate_command!(get_songs_by_options, Database, Vec<Song>, options: GetSongOptions);
generate_command!(get_entity_by_options, Database, Value, options: GetEntityOptions);
generate_command!(search_all, Database, SearchResult, term: String);
generate_command!(create_playlist, Database, String, playlist: Playlist);
generate_command!(add_to_playlist, Database, (), id: String, songs: Vec<Song>);
generate_command!(is_song_in_playlist, Database, bool, playlist_id: String, song_id: String);
generate_command!(remove_from_playlist, Database, (), id: String, songs: Vec<String>);
generate_command!(remove_playlist, Database, (), id: String);
generate_command!(update_album, Database, (), album: Album);
generate_command!(update_artist, Database, (), artist: Artist);
generate_command!(update_playlist, Database, (), playlist: Playlist);
generate_command!(update_songs, Database, (), songs: Vec<Song>);
generate_command!(update_lyrics, Database, (), id: String, lyrics: String);
generate_command!(increment_play_count, Database, (), id: String);
generate_command!(increment_play_time, Database, (), id: String, duration: f64);
generate_command!(get_top_listened_songs, Database, AllAnalytics,);

#[tracing::instrument(level = "debug", skip(app))]
pub fn get_cache_state(app: &mut App) -> CacheHolder {
    let path = app.path().app_cache_dir().unwrap().join("http_cache.db");
    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent).unwrap();
    }

    CacheHolder::new(path)
}

#[tracing::instrument(level = "debug", skip(app))]
pub fn get_db_state(app: &mut App) -> Database {
    let path = app.path().app_data_dir().unwrap().join("songs.db");
    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent).unwrap();
    }

    Database::new(path)
}
