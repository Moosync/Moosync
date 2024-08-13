use std::hash;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    entities::{QueryableAlbum, QueryableArtist, QueryablePlaylist},
    preferences::PreferenceUIData,
    songs::Song,
    ui::player_details::PlayerState,
};

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GenericExtensionHostRequest<T: Serialize> {
    #[serde(rename = "type")]
    pub type_: String,
    pub channel: String,
    pub data: Option<T>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PackageNameArgs {
    pub package_name: String,
}

impl From<String> for PackageNameArgs {
    fn from(value: String) -> Self {
        Self {
            package_name: value,
        }
    }
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]

pub struct ToggleExtArgs {
    pub package_name: String,
    pub toggle: bool,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextMenuActionArgs {
    pub package_name: String,
    pub id: String,
    pub arg: Value,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccountLoginArgs {
    pub package_name: String,
    pub account_id: String,
    pub login_status: bool,
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
    // TODO: Use a concrete type for this
    pub preferences: Vec<PreferenceUIData>,
    pub extension_path: String,
    pub extension_icon: Option<String>,
}

impl hash::Hash for ExtensionDetail {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.package_name.hash(state)
    }
}

impl PartialEq for ExtensionDetail {
    fn eq(&self, other: &Self) -> bool {
        self.package_name == other.package_name
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionContextMenuItem {
    #[serde(rename = "type")]
    pub type_: String,
    pub label: String,
    pub disabled: bool,
    pub children: Vec<ExtensionContextMenuItem>,
    pub handler: String,
}

#[derive(Deserialize, Debug, Clone)]
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

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ExtensionProviderScope {
    Search,
    Playlists,
    PlaylistSongs,
    ArtistSongs,
    AlbumSongs,
    Recommendations,
    Scrobbles,
    PlaylistFromUrl,
    SongFromUrl,
    SearchAlbum,
    SearchArtist,
}

#[derive(Serialize, Debug, Clone)]
pub struct PreferenceArgs {
    key: String,
    value: Value,
}

#[derive(Serialize, Debug, Clone)]
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
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionExtraEventArgs {
    #[serde(flatten)]
    pub data: ExtensionExtraEvent,
    pub package_name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistReturnType {
    pub playlists: Vec<QueryablePlaylist>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongsReturnType {
    pub songs: Vec<Song>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongsWithPageTokenReturnType {
    pub songs: Vec<Song>,
    pub next_page_token: Option<serde_json::Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SearchReturnType {
    pub songs: Vec<Song>,
    pub playlists: Vec<QueryablePlaylist>,
    pub artists: Vec<QueryableArtist>,
    pub albums: Vec<QueryableAlbum>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackDetailsReturnType {
    pub duration: u32,
    pub url: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CustomRequestReturnType {
    pub mime_type: Option<String>,
    pub data: Option<Vec<u8>>, // Buffer is typically represented as Vec<u8> in Rust
    pub redirect_url: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongReturnType {
    pub song: Song,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistAndSongsReturnType {
    pub playlist: QueryablePlaylist,
    pub songs: Vec<Song>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationsReturnType {
    pub songs: Vec<Song>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionUIRequest {
    #[serde(rename = "type")]
    pub type_: String,
    pub extension_name: String,
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
    pub release: Release,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Release {
    pub r#type: Option<String>,
    pub url: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionManifest {
    pub moosync_extension: bool,
    pub display_name: String,
    pub extension_entry: String,
    pub name: String,
    pub version: String,
}
