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

use std::hash;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    entities::{QueryableAlbum, QueryableArtist, QueryablePlaylist},
    preferences::PreferenceUIData,
    songs::Song,
};

use super::player_details::PlayerState;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionUIRequest {
    #[serde(rename = "type")]
    pub type_: String,
    pub data: Option<Value>,
    pub channel: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FetchedExtensionManifest {
    pub name: String,
    pub package_name: String,
    pub logo: Option<String>,
    pub description: Option<String>,
    pub url: String,
    pub version: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionDetail {
    pub name: String,
    pub package_name: String,
    pub desc: Option<String>,
    pub author: Option<String>,
    pub version: String,
    pub has_started: bool,
    pub entry: String,
    pub preferences: Vec<PreferenceUIData>,
    pub extension_path: String,
    pub extension_icon: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccountLoginArgs {
    pub package_name: String,
    pub account_id: String,
    pub login_status: bool,
}

impl hash::Hash for ExtensionDetail {
    #[tracing::instrument(level = "trace", skip(self, state))]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.package_name.hash(state)
    }
}

impl PartialEq for ExtensionDetail {
    #[tracing::instrument(level = "trace", skip(self, other))]
    fn eq(&self, other: &Self) -> bool {
        self.package_name == other.package_name
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionContextMenuItem {
    #[serde(rename = "type")]
    pub type_: String,
    pub label: String,
    pub disabled: bool,
    pub children: Vec<ExtensionContextMenuItem>,
    pub handler: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionAccountDetail {
    pub id: String,
    pub package_name: String,
    pub name: String,
    pub bg_color: String,
    pub icon: String,
    pub logged_in: bool,
    pub username: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PreferenceArgs {
    pub key: String,
    pub value: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", tag = "type", content = "data")]
pub enum ExtensionExtraEvent {
    RequestedPlaylists([bool; 1]),
    RequestedPlaylistSongs(String, bool, Option<String>),
    OauthCallback([String; 1]),
    SongQueueChanged([Value; 1]),
    Seeked([f64; 1]),
    VolumeChanged([f64; 1]),
    PlayerStateChanged([PlayerState; 1]),
    SongChanged([Option<Song>; 1]),
    PreferenceChanged([PreferenceArgs; 1]),
    PlaybackDetailsRequested([Song; 1]),
    CustomRequest([String; 1]),
    RequestedSongFromURL(String, bool),
    RequestedPlaylistFromURL(String, bool),
    RequestedSearchResult([String; 1]),
    RequestedRecommendations,
    RequestedLyrics([Song; 1]),
    RequestedArtistSongs(QueryableArtist, Option<String>),
    RequestedAlbumSongs(QueryableAlbum, Option<String>),
    SongAdded([Vec<Song>; 1]),
    SongRemoved([Vec<Song>; 1]),
    PlaylistAdded([Vec<QueryablePlaylist>; 1]),
    PlaylistRemoved([Vec<QueryablePlaylist>; 1]),
    RequestedSongFromId([String; 1]),
    GetRemoteURL([Song; 1]),
    Scrobble([Song; 1]),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionExtraEventArgs {
    #[serde(flatten)]
    pub data: ExtensionExtraEvent,
    pub package_name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistReturnType {
    pub playlists: Vec<QueryablePlaylist>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SongsReturnType {
    pub songs: Vec<Song>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SongsWithPageTokenReturnType {
    pub songs: Vec<Song>,
    pub next_page_token: Option<serde_json::Value>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SearchReturnType {
    pub songs: Vec<Song>,
    pub playlists: Vec<QueryablePlaylist>,
    pub artists: Vec<QueryableArtist>,
    pub albums: Vec<QueryableAlbum>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackDetailsReturnType {
    pub duration: u32,
    pub url: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CustomRequestReturnType {
    pub mime_type: Option<String>,
    pub data: Option<Vec<u8>>, // Buffer is typically represented as Vec<u8> in Rust
    pub redirect_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SongReturnType {
    pub song: Option<Song>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistAndSongsReturnType {
    pub playlist: Option<QueryablePlaylist>,
    pub songs: Option<Vec<Song>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationsReturnType {
    pub songs: Vec<Song>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddToPlaylistRequest {
    #[serde(rename = "playlistID")]
    pub playlist_id: String,
    pub songs: Vec<Song>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PreferenceData {
    pub key: String,
    pub value: Option<Value>,
    #[serde(rename = "defaultValue")]
    pub default_value: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PackageNameArgs {
    pub package_name: String,
}

impl From<String> for PackageNameArgs {
    #[tracing::instrument(level = "trace", skip(value))]
    fn from(value: String) -> Self {
        Self {
            package_name: value,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]

pub struct ToggleExtArgs {
    pub package_name: String,
    pub toggle: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextMenuActionArgs {
    pub package_name: String,
    pub id: String,
    pub arg: Value,
}
