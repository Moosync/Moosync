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

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::errors::Result as MoosyncResult;
use crate::{
    entities::{Album, Artist, Genre, GetEntityOptions, Playlist},
    preferences::PreferenceUIData,
    songs::{GetSongOptions, Song},
    ui::{
        extensions::{AddToPlaylistRequest, ExtensionUIRequest, PreferenceData},
        player_details::PlayerState,
    },
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

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum MainCommand {
    GetSong(GetSongOptions),
    GetEntity(GetEntityOptions),
    GetCurrentSong(),
    GetPlayerState(),
    GetVolume(),
    GetTime(),
    GetQueue(),
    GetPreference(PreferenceData),
    SetPreference(PreferenceData),
    GetSecure(PreferenceData),
    SetSecure(PreferenceData),
    AddSongs(Vec<Song>),
    RemoveSong(Song),
    UpdateSong(Song),
    AddPlaylist(Playlist),
    AddToPlaylist(AddToPlaylistRequest),
    RegisterOAuth(String),
    OpenExternalUrl(String),
    UpdateAccounts(Option<String>),
    RegisterUserPreference(Vec<PreferenceUIData>),
    UnregisterUserPreference(Vec<String>),
    ExtensionsUpdated(),
    GetAppVersion(),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum MainCommandResponse {
    GetSong(Vec<Song>),
    GetEntity(Value),
    GetCurrentSong(Option<Song>),
    GetPlayerState(PlayerState),
    GetVolume(f64),
    GetTime(f64),
    GetQueue(Value),
    GetPreference(PreferenceData),
    SetPreference(bool),
    GetSecure(PreferenceData),
    SetSecure(bool),
    AddSongs(Vec<Song>),
    RemoveSong(bool),
    UpdateSong(Song),
    AddPlaylist(String),
    AddToPlaylist(bool),
    RegisterOAuth(bool),
    OpenExternalUrl(bool),
    UpdateAccounts(bool),
    RegisterUserPreference(bool),
    UnregisterUserPreference(bool),
    ExtensionsUpdated(bool),
    GetAppVersion(String),
    Error(String),
}

impl MainCommand {
    pub fn sanitize_command(&mut self, package_name: &str) {
        match self {
            MainCommand::GetPreference(preference_data) => {
                preference_data.key = format!("extensions.{}", preference_data.key);
            }
            MainCommand::SetPreference(preference_data) => {
                preference_data.key = format!("extensions.{}", preference_data.key);
            }
            MainCommand::GetSecure(preference_data) => {
                preference_data.key =
                    format!("extensions.{}.{}", package_name, preference_data.key);
            }
            MainCommand::SetSecure(preference_data) => {
                preference_data.key =
                    format!("extensions.{}.{}", package_name, preference_data.key);
            }
            MainCommand::AddSongs(songs) => {
                let prefix = format!("{}:", package_name);
                for song in songs {
                    sanitize_song(&prefix, song);
                }
            }
            MainCommand::RemoveSong(song) => {
                let prefix = format!("{}:", package_name);
                sanitize_song(&prefix, song);
            }
            MainCommand::UpdateSong(song) => {
                let prefix = format!("{}:", package_name);
                sanitize_song(&prefix, song);
            }
            MainCommand::AddPlaylist(queryable_playlist) => {
                let prefix = format!("{}:", package_name);
                sanitize_playlist(&prefix, queryable_playlist);
            }
            MainCommand::AddToPlaylist(add_to_playlist_request) => {
                let prefix = format!("{}:", package_name);
                for song in add_to_playlist_request.songs.iter_mut() {
                    sanitize_song(&prefix, song);
                }
            }
            MainCommand::RegisterOAuth(_) => todo!(),
            MainCommand::OpenExternalUrl(_) => todo!(),
            MainCommand::UpdateAccounts(package_name_inner) => {
                package_name_inner.replace(package_name.to_string());
            }
            _ => {}
        }
    }

    pub fn to_ui_request(&mut self) -> MoosyncResult<ExtensionUIRequest> {
        let (r#type, data) = match self {
            MainCommand::GetCurrentSong() => ("getCurrentSong", Value::Null),
            MainCommand::GetPlayerState() => ("getPlayerState", Value::Null),
            MainCommand::GetVolume() => ("getVolume", Value::Null),
            MainCommand::GetTime() => ("getTime", Value::Null),
            MainCommand::GetQueue() => ("getQueue", Value::Null),
            _ => unreachable!("Any other request should not have been sent as UI request"),
        };

        Ok(ExtensionUIRequest {
            type_: r#type.into(),
            channel: "".into(),
            data: Some(data),
        })
    }
}
