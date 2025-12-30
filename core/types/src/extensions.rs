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

use std::fmt::Debug;

use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::{
    entities::{Album, Artist, Genre, Playlist},
    songs::Song,
};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ManifestPermissions {
    pub hosts: Vec<String>,
    pub paths: HashMap<String, PathBuf>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionManifest {
    pub moosync_extension: bool,
    pub display_name: String,
    pub extension_entry: PathBuf,
    pub author: Option<String>,
    pub name: String,
    pub version: String,
    pub icon: String,
    pub permissions: Option<ManifestPermissions>,
}

pub fn sanitize_album(prefix: &str, album: &mut Album) {
    if let Some(id) = album.album_id.as_mut()
        && !id.starts_with(prefix)
    {
        *id = format!("{}{}", prefix, id);
    }
}

pub fn sanitize_artist(prefix: &str, artist: &mut Artist) {
    if let Some(id) = artist.artist_id.as_mut()
        && !id.starts_with(prefix)
    {
        *id = format!("{}{}", prefix, id);
    }
}

pub fn sanitize_genre(prefix: &str, genre: &mut Genre) {
    if let Some(id) = genre.genre_id.as_mut()
        && !id.starts_with(prefix)
    {
        *id = format!("{}{}", prefix, id);
    }
}

pub fn sanitize_song(prefix: &str, song: &mut Song) {
    if let Some(id) = song.song._id.as_mut()
        && !id.starts_with(prefix)
    {
        *id = format!("{}{}", prefix, id);
    }

    if let Some(album) = song.album.as_mut() {
        sanitize_album(prefix, album);
    }

    if let Some(artists) = song.artists.as_mut() {
        artists.iter_mut().for_each(|a| sanitize_artist(prefix, a));
    }

    if let Some(genre) = song.genre.as_mut() {
        genre.iter_mut().for_each(|a| sanitize_genre(prefix, a));
    }
}

pub fn sanitize_playlist(prefix: &str, playlist: &mut Playlist) {
    if let Some(playlist_id) = playlist.playlist_id.as_mut()
        && !playlist_id.starts_with(prefix)
    {
        *playlist_id = format!("{}{}", prefix, playlist_id);
    }
}
