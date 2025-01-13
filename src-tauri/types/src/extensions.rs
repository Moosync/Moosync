use std::fmt::Debug;

use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

use extism_convert::{FromBytes, Json, ToBytes};

use crate::{
    entities::{
        GetEntityOptions, QueryableAlbum, QueryableArtist, QueryableGenre, QueryablePlaylist,
    },
    errors::{MoosyncError, Result as MoosyncResult},
    songs::{GetSongOptions, Song},
    ui::{
        extensions::{
            AccountLoginArgs, AddToPlaylistRequest, CustomRequestReturnType,
            ExtensionAccountDetail, ExtensionContextMenuItem, ExtensionDetail, ExtensionExtraEvent,
            ExtensionExtraEventArgs, ExtensionUIRequest, PackageNameArgs,
            PlaybackDetailsReturnType, PlaylistAndSongsReturnType, PlaylistReturnType,
            PreferenceData, RecommendationsReturnType, SearchReturnType, SongReturnType,
            SongsWithPageTokenReturnType,
        },
        player_details::PlayerState,
    },
};

#[derive(Debug, Clone)]
pub struct GenericExtensionHostRequest<T: Clone + Debug> {
    pub channel: String,
    pub data: Option<T>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
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
    PlaybackDetails,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct EmptyResp {}

#[derive(Debug, Serialize, Deserialize, ToBytes, FromBytes)]
#[encoding(Json)]
pub struct ExtensionDetailsWrapper(pub ExtensionDetail);

#[derive(Debug, Serialize, Deserialize, ToBytes, FromBytes)]
#[encoding(Json)]
pub struct JsonWrapper<T>(pub T);

#[derive(Debug, Deserialize, Serialize, FromBytes, ToBytes, Clone)]
#[encoding(Json)]
#[serde(untagged)]
pub enum ExtensionExtraEventResponse {
    RequestedPlaylists(PlaylistReturnType),
    RequestedPlaylistSongs(SongsWithPageTokenReturnType),
    OauthCallback,
    SongQueueChanged,
    Seeked,
    VolumeChanged,
    PlayerStateChanged,
    SongChanged,
    PreferenceChanged,
    PlaybackDetailsRequested(PlaybackDetailsReturnType),
    CustomRequest(CustomRequestReturnType),
    RequestedSongFromURL(SongReturnType),
    RequestedPlaylistFromURL(PlaylistAndSongsReturnType),
    RequestedSearchResult(SearchReturnType),
    RequestedRecommendations(RecommendationsReturnType),
    RequestedLyrics(String),
    RequestedArtistSongs(SongsWithPageTokenReturnType),
    RequestedAlbumSongs(SongsWithPageTokenReturnType),
    SongAdded,
    SongRemoved,
    PlaylistAdded,
    PlaylistRemoved,
    RequestedSongFromId(SongReturnType),
    GetRemoteURL(String),
    Scrobble,
}

#[tracing::instrument(level = "trace", skip(field))]
fn serialize_null<S>(field: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    field.serialize_none()
}

#[derive(Debug, Deserialize, Serialize, FromBytes, ToBytes, Clone)]
#[serde(untagged)]
#[encoding(Json)]
pub enum ExtensionCommandResponse {
    GetProviderScopes(Vec<ExtensionProviderScope>),
    GetExtensionContextMenu(Vec<ExtensionContextMenuItem>),
    GetAccounts(Vec<ExtensionAccountDetail>),
    PerformAccountLogin,
    ExtraExtensionEvent(Box<ExtensionExtraEventResponse>),

    #[serde(serialize_with = "serialize_null")]
    Empty,
}

#[derive(Debug, Clone)]
pub enum ExtensionCommand {
    GetProviderScopes(PackageNameArgs),
    GetExtensionContextMenu(PackageNameArgs),
    GetAccounts(PackageNameArgs),
    PerformAccountLogin(AccountLoginArgs),
    ExtraExtensionEvent(ExtensionExtraEventArgs),
}

impl TryFrom<(&str, &Value)> for ExtensionCommand {
    type Error = MoosyncError;
    #[tracing::instrument(level = "trace", skip())]
    fn try_from((r#type, data): (&str, &Value)) -> std::result::Result<Self, Self::Error> {
        match r#type {
            "extraExtensionEvents" => {
                let res = serde_json::from_value(data.clone());
                if let Ok(res) = res {
                    return Ok(ExtensionCommand::ExtraExtensionEvent(res));
                }
            }
            "getExtensionProviderScopes" => {
                let res = serde_json::from_value(data.clone());
                if let Ok(res) = res {
                    return Ok(ExtensionCommand::GetProviderScopes(res));
                }
            }
            "getExtensionContextMenu" => {
                let res = serde_json::from_value(data.clone());
                if let Ok(res) = res {
                    return Ok(ExtensionCommand::GetExtensionContextMenu(res));
                }
            }
            "getAccounts" => {
                let res = serde_json::from_value(data.clone());
                if let Ok(res) = res {
                    return Ok(ExtensionCommand::GetAccounts(res));
                }
            }
            "performAccountLogin" => {
                let res = serde_json::from_value(data.clone());
                if let Ok(res) = res {
                    return Ok(ExtensionCommand::PerformAccountLogin(res));
                }
            }
            _ => {}
        }
        Err("Invalid command".into())
    }
}

impl ExtensionCommand {
    #[tracing::instrument(level = "trace", skip(self))]
    pub fn to_plugin_call(&self) -> (String, &'static str, Vec<u8>) {
        match self {
            Self::GetProviderScopes(args) => (
                args.package_name.clone(),
                "get_provider_scopes_wrapper",
                vec![],
            ),
            Self::GetExtensionContextMenu(args) => (
                args.package_name.clone(),
                "get_context_menu_wrapper",
                vec![],
            ),
            Self::GetAccounts(args) => (args.package_name.clone(), "get_accounts_wrapper", vec![]),
            Self::PerformAccountLogin(args) => (
                args.package_name.clone(),
                "perform_account_login_wrapper",
                Json(args).to_bytes().unwrap(),
            ),
            Self::ExtraExtensionEvent(args) => {
                let package_name = args.package_name.clone();
                let res = match &args.data {
                    ExtensionExtraEvent::RequestedPlaylists(_) => ("get_playlists_wrapper", vec![]),
                    ExtensionExtraEvent::RequestedPlaylistSongs(id, _, token) => (
                        "get_playlist_content_wrapper",
                        Json((id, token)).to_bytes().unwrap(),
                    ),
                    ExtensionExtraEvent::OauthCallback(code) => (
                        "oauth_callback_wrapper",
                        Json(code[0].clone()).to_bytes().unwrap(),
                    ),
                    ExtensionExtraEvent::SongQueueChanged(value) => (
                        "on_queue_changed_wrapper",
                        Json(value[0].clone()).to_bytes().unwrap(),
                    ),
                    ExtensionExtraEvent::Seeked(time) => {
                        ("on_seeked_wrapper", Json(time[0]).to_bytes().unwrap())
                    }
                    ExtensionExtraEvent::VolumeChanged(_) => ("on_volume_changed_wrapper", vec![]),
                    ExtensionExtraEvent::PlayerStateChanged(player_state) => (
                        "on_player_state_changed_wrapper",
                        Json(player_state[0]).to_bytes().unwrap(),
                    ),
                    ExtensionExtraEvent::SongChanged(_) => ("on_song_changed_wrapper", vec![]),
                    ExtensionExtraEvent::PreferenceChanged(preferences) => (
                        "on_preferences_changed_wrapper",
                        Json(preferences[0].clone()).to_bytes().unwrap(),
                    ),
                    ExtensionExtraEvent::PlaybackDetailsRequested(songs) => (
                        "get_playback_details_wrapper",
                        Json(songs[0].clone()).to_bytes().unwrap(),
                    ),
                    ExtensionExtraEvent::CustomRequest(url) => (
                        "handle_custom_request_wrapper",
                        Json(url[0].clone()).to_bytes().unwrap(),
                    ),
                    ExtensionExtraEvent::RequestedSongFromURL(url, _) => {
                        ("get_song_from_url_wrapper", Json(url).to_bytes().unwrap())
                    }
                    ExtensionExtraEvent::RequestedPlaylistFromURL(url, _) => (
                        "get_playlist_from_url_wrapper",
                        Json(url).to_bytes().unwrap(),
                    ),
                    ExtensionExtraEvent::RequestedSearchResult(term) => {
                        ("search_wrapper", Json(term[0].clone()).to_bytes().unwrap())
                    }
                    ExtensionExtraEvent::RequestedRecommendations => {
                        ("get_recommendations_wrapper", vec![])
                    }
                    ExtensionExtraEvent::RequestedLyrics(_) => todo!(),
                    ExtensionExtraEvent::RequestedArtistSongs(artist, token) => (
                        "get_artist_songs_wrapper",
                        Json((artist.clone(), token)).to_bytes().unwrap(),
                    ),
                    ExtensionExtraEvent::RequestedAlbumSongs(album, token) => (
                        "get_album_songs_wrapper",
                        Json((album.clone(), token)).to_bytes().unwrap(),
                    ),
                    ExtensionExtraEvent::SongAdded(song) => (
                        "on_song_added_wrapper",
                        Json(song[0].clone()).to_bytes().unwrap(),
                    ),
                    ExtensionExtraEvent::SongRemoved(song) => (
                        "on_song_removed_wrapper",
                        Json(song[0].clone()).to_bytes().unwrap(),
                    ),
                    ExtensionExtraEvent::PlaylistAdded(playlist) => (
                        "on_playlist_added_wrapper",
                        Json(playlist[0].clone()).to_bytes().unwrap(),
                    ),
                    ExtensionExtraEvent::PlaylistRemoved(playlist) => (
                        "on_playlist_removed_wrapper",
                        Json(playlist[0].clone()).to_bytes().unwrap(),
                    ),
                    ExtensionExtraEvent::RequestedSongFromId(id) => (
                        "get_song_from_id_wrapper",
                        Json(id[0].clone()).to_bytes().unwrap(),
                    ),
                    ExtensionExtraEvent::GetRemoteURL(_) => ("get_remote_url_wrapper", vec![]),
                    ExtensionExtraEvent::Scrobble(song) => (
                        "scrobble_wrapper",
                        Json(song[0].clone()).to_bytes().unwrap(),
                    ),
                };
                (package_name, res.0, res.1)
            }
        }
    }

    #[tracing::instrument(level = "trace", skip(self, value))]
    pub fn parse_response(&self, value: Value) -> MoosyncResult<ExtensionCommandResponse> {
        let ret = match self {
            Self::GetProviderScopes(_) => {
                ExtensionCommandResponse::GetProviderScopes(serde_json::from_value(value)?)
            }
            Self::GetExtensionContextMenu(_) => ExtensionCommandResponse::GetExtensionContextMenu(
                serde_json::from_value(value).unwrap(),
            ),
            Self::GetAccounts(_) => {
                ExtensionCommandResponse::GetAccounts(serde_json::from_value(value)?)
            }
            Self::PerformAccountLogin(_) => ExtensionCommandResponse::PerformAccountLogin,
            Self::ExtraExtensionEvent(args) => {
                let res = match &args.data {
                    ExtensionExtraEvent::RequestedPlaylists(_) => {
                        ExtensionExtraEventResponse::RequestedPlaylists(serde_json::from_value(
                            value,
                        )?)
                    }
                    ExtensionExtraEvent::RequestedPlaylistSongs(_, _, _) => {
                        ExtensionExtraEventResponse::RequestedPlaylistSongs(serde_json::from_value(
                            value,
                        )?)
                    }
                    ExtensionExtraEvent::OauthCallback(_) => {
                        ExtensionExtraEventResponse::OauthCallback
                    }
                    ExtensionExtraEvent::SongQueueChanged(_) => {
                        ExtensionExtraEventResponse::SongQueueChanged
                    }
                    ExtensionExtraEvent::Seeked(_) => ExtensionExtraEventResponse::Seeked,
                    ExtensionExtraEvent::VolumeChanged(_) => {
                        ExtensionExtraEventResponse::VolumeChanged
                    }
                    ExtensionExtraEvent::PlayerStateChanged(_) => {
                        ExtensionExtraEventResponse::PlayerStateChanged
                    }
                    ExtensionExtraEvent::SongChanged(_) => ExtensionExtraEventResponse::SongChanged,
                    ExtensionExtraEvent::PreferenceChanged(_) => {
                        ExtensionExtraEventResponse::PreferenceChanged
                    }
                    ExtensionExtraEvent::PlaybackDetailsRequested(_) => {
                        ExtensionExtraEventResponse::PlaybackDetailsRequested(
                            serde_json::from_value(value)?,
                        )
                    }
                    ExtensionExtraEvent::CustomRequest(_) => {
                        ExtensionExtraEventResponse::CustomRequest(serde_json::from_value(value)?)
                    }
                    ExtensionExtraEvent::RequestedSongFromURL(_, _) => {
                        ExtensionExtraEventResponse::RequestedSongFromURL(serde_json::from_value(
                            value,
                        )?)
                    }
                    ExtensionExtraEvent::RequestedPlaylistFromURL(_, _) => {
                        ExtensionExtraEventResponse::RequestedPlaylistFromURL(
                            serde_json::from_value(value)?,
                        )
                    }
                    ExtensionExtraEvent::RequestedSearchResult(_) => {
                        ExtensionExtraEventResponse::RequestedSearchResult(serde_json::from_value(
                            value,
                        )?)
                    }
                    ExtensionExtraEvent::RequestedRecommendations => {
                        ExtensionExtraEventResponse::RequestedRecommendations(
                            serde_json::from_value(value)?,
                        )
                    }
                    ExtensionExtraEvent::RequestedLyrics(_) => {
                        ExtensionExtraEventResponse::RequestedLyrics(serde_json::from_value(value)?)
                    }
                    ExtensionExtraEvent::RequestedArtistSongs(_, _) => {
                        ExtensionExtraEventResponse::RequestedArtistSongs(serde_json::from_value(
                            value,
                        )?)
                    }
                    ExtensionExtraEvent::RequestedAlbumSongs(_, _) => {
                        ExtensionExtraEventResponse::RequestedAlbumSongs(serde_json::from_value(
                            value,
                        )?)
                    }
                    ExtensionExtraEvent::SongAdded(_) => ExtensionExtraEventResponse::SongAdded,
                    ExtensionExtraEvent::SongRemoved(_) => ExtensionExtraEventResponse::SongRemoved,
                    ExtensionExtraEvent::PlaylistAdded(_) => {
                        ExtensionExtraEventResponse::PlaylistAdded
                    }
                    ExtensionExtraEvent::PlaylistRemoved(_) => {
                        ExtensionExtraEventResponse::PlaylistRemoved
                    }
                    ExtensionExtraEvent::RequestedSongFromId(_) => {
                        ExtensionExtraEventResponse::RequestedSongFromId(serde_json::from_value(
                            value,
                        )?)
                    }
                    ExtensionExtraEvent::GetRemoteURL(_) => {
                        ExtensionExtraEventResponse::GetRemoteURL(serde_json::from_value(value)?)
                    }
                    ExtensionExtraEvent::Scrobble(_) => ExtensionExtraEventResponse::Scrobble,
                };
                ExtensionCommandResponse::ExtraExtensionEvent(Box::new(res))
            }
        };
        Ok(ret)
    }
}

#[derive(Debug)]
pub enum RunnerCommand {
    FindNewExtensions,
    GetInstalledExtensions,
    GetExtensionIcon(PackageNameArgs),
    ToggleExtensionStatus(PackageNameArgs),
    RemoveExtension(PackageNameArgs),
    StopProcess,
    GetDisplayName(PackageNameArgs),
}

impl TryFrom<(&str, &Value)> for RunnerCommand {
    type Error = MoosyncError;

    #[tracing::instrument(level = "trace", skip())]
    fn try_from((r#type, data): (&str, &Value)) -> std::result::Result<Self, Self::Error> {
        match r#type {
            "findNewExtensions" => Ok(Self::FindNewExtensions),
            "getInstalledExtensions" => Ok(Self::GetInstalledExtensions),
            "getExtensionIcon" => Ok(Self::GetExtensionIcon(
                serde_json::from_value(data.clone()).unwrap(),
            )),
            "toggleExtensionStatus" => Ok(Self::ToggleExtensionStatus(
                serde_json::from_value(data.clone()).unwrap(),
            )),
            "removeExtension" => Ok(Self::RemoveExtension(
                serde_json::from_value(data.clone()).unwrap(),
            )),
            "stopProcess" => Ok(Self::StopProcess),
            "getDisplayName" => Ok(Self::GetDisplayName(
                serde_json::from_value(data.clone()).unwrap(),
            )),
            _ => Err("Failed to parse runner command".into()),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestPermissions {
    pub hosts: Vec<String>,
    pub paths: HashMap<String, PathBuf>,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize, Serialize, ToBytes, FromBytes, Clone)]
#[encoding(Json)]
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
    AddPlaylist(QueryablePlaylist),
    AddToPlaylist(AddToPlaylistRequest),
    RegisterOAuth(String),
    OpenExternalUrl(String),
    UpdateAccounts(Option<String>),
    ExtensionsUpdated(),
}

#[derive(Debug, Deserialize, Serialize, ToBytes, FromBytes, Clone)]
#[encoding(Json)]
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
    ExtensionsUpdated(bool),
}

impl MainCommand {
    #[cfg(not(feature = "extensions"))]
    #[tracing::instrument(level = "trace", skip(self))]
    pub fn to_request(&self) -> MoosyncResult<GenericExtensionHostRequest<MainCommand>> {
        Ok(GenericExtensionHostRequest {
            channel: uuid::Uuid::new_v4().to_string(),
            data: Some(self.clone()),
        })
    }

    #[cfg(not(feature = "extensions"))]
    pub fn to_ui_request(&mut self) -> MoosyncResult<ExtensionUIRequest> {
        let (r#type, data) = match self {
            MainCommand::GetSong(options) => ("getSongs", serde_json::to_value(options)?),
            MainCommand::GetEntity(options) => ("getEntity", serde_json::to_value(options)?),
            MainCommand::GetCurrentSong() => ("getCurrentSong", Value::Null),
            MainCommand::GetPlayerState() => ("getPlayerState", Value::Null),
            MainCommand::GetVolume() => ("getVolume", Value::Null),
            MainCommand::GetTime() => ("getTime", Value::Null),
            MainCommand::GetQueue() => ("getQueue", Value::Null),
            MainCommand::GetPreference(options) => (
                "getPreferences",
                serde_json::to_value(sanitize_prefs(options))?,
            ),
            MainCommand::SetPreference(options) => (
                "setPreferences",
                serde_json::to_value(sanitize_prefs(options))?,
            ),
            MainCommand::GetSecure(options) => (
                "getSecurePreferences",
                serde_json::to_value(sanitize_prefs(options))?,
            ),
            MainCommand::SetSecure(options) => (
                "setSecurePreferences",
                serde_json::to_value(sanitize_prefs(options))?,
            ),
            MainCommand::AddSongs(songs) => ("addSong", serde_json::to_value(songs)?),
            MainCommand::RemoveSong(song) => ("removeSong", serde_json::to_value(song)?),
            MainCommand::UpdateSong(song) => ("updateSong", serde_json::to_value(song)?),
            MainCommand::AddPlaylist(playlist) => ("addPlaylist", serde_json::to_value(playlist)?),
            MainCommand::AddToPlaylist(options) => {
                ("addToPlaylist", serde_json::to_value(options)?)
            }
            MainCommand::RegisterOAuth(url) => ("registerOauth", Value::String(url.clone())),
            MainCommand::OpenExternalUrl(url) => ("openExternal", Value::String(url.clone())),
            MainCommand::UpdateAccounts(key) => ("updateAccounts", serde_json::to_value(key)?),
            MainCommand::ExtensionsUpdated() => ("extensionsUpdated", Value::Null),
        };

        Ok(ExtensionUIRequest {
            type_: r#type.into(),
            channel: uuid::Uuid::new_v4().to_string(),
            data: Some(data),
        })
    }
}

pub fn sanitize_album(prefix: &str, album: &mut QueryableAlbum) {
    if let Some(id) = album.album_id.as_mut() {
        if !id.starts_with(prefix) {
            *id = format!("{}{}", prefix, id);
        }
    }
}

pub fn sanitize_artist(prefix: &str, artist: &mut QueryableArtist) {
    if let Some(id) = artist.artist_id.as_mut() {
        if !id.starts_with(prefix) {
            *id = format!("{}{}", prefix, id);
        }
    }
}

pub fn sanitize_genre(prefix: &str, genre: &mut QueryableGenre) {
    if let Some(id) = genre.genre_id.as_mut() {
        if !id.starts_with(prefix) {
            *id = format!("{}{}", prefix, id);
        }
    }
}

pub fn sanitize_song(prefix: &str, song: &mut Song) {
    if let Some(id) = song.song._id.as_mut() {
        if !id.starts_with(prefix) {
            *id = format!("{}{}", prefix, id);
        }
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

pub fn sanitize_playlist(prefix: &str, playlist: &mut QueryablePlaylist) {
    if let Some(playlist_id) = playlist.playlist_id.as_mut() {
        if !playlist_id.starts_with(prefix) {
            *playlist_id = format!("{}{}", prefix, playlist_id);
        }
    }
}

fn sanitize_prefs(options: &PreferenceData) -> PreferenceData {
    PreferenceData {
        key: format!("extensions.{}", options.key),
        ..options.clone()
    }
}

#[derive(Debug)]
pub enum RunnerCommandResp {
    ExtensionList(Vec<ExtensionDetail>),
    ExtensionIcon(Option<String>),
    ExtensionName(Option<String>),
    Empty(),
}
