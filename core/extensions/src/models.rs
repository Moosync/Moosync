use std::fmt::Debug;

use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;
use types::{
    errors::{MoosyncError, Result as MoosyncResult},
    extensions::{sanitize_album, sanitize_artist, sanitize_playlist, sanitize_song},
    ui::extensions::{
        AccountLoginArgs, ContextMenuReturnType, CustomRequestReturnType, ExtensionAccountDetail,
        ExtensionExtraEvent, ExtensionExtraEventArgs, ExtensionProviderScope, PackageNameArgs,
        PlaybackDetailsReturnType, PlaylistAndSongsReturnType, PlaylistReturnType,
        RecommendationsReturnType, SearchReturnType, SongReturnType, SongsWithPageTokenReturnType,
    },
};

#[derive(Debug, Clone)]
pub struct GenericExtensionHostRequest<T: Clone + Debug> {
    pub package_name: String,
    pub channel: String,
    pub data: Option<T>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
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
    RequestedSongContextMenu(Vec<ContextMenuReturnType>),
    RequestedPlaylistContextMenu(Vec<ContextMenuReturnType>),
    ContextMenuAction,
}

#[tracing::instrument(level = "debug", skip(field))]
fn serialize_null<S>(field: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    field.serialize_none()
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum ExtensionCommandResponse {
    GetProviderScopes(Vec<ExtensionProviderScope>),
    GetAccounts(Vec<ExtensionAccountDetail>),
    PerformAccountLogin(String),
    ExtraExtensionEvent(Box<ExtensionExtraEventResponse>),

    #[serde(serialize_with = "serialize_null")]
    Empty,
}

impl ExtensionCommandResponse {
    pub fn sanitize(&mut self, package_name: &str) {
        match self {
            ExtensionCommandResponse::GetProviderScopes(_) => {}
            ExtensionCommandResponse::GetAccounts(accounts) => {
                for account in accounts {
                    account.package_name = package_name.to_string();
                }
            }
            ExtensionCommandResponse::PerformAccountLogin(_) => {}
            ExtensionCommandResponse::ExtraExtensionEvent(resp) => {
                let prefix = format!("{}:", package_name);
                let resp = resp.as_mut();
                match resp {
                    ExtensionExtraEventResponse::RequestedPlaylists(playlist_return_type) => {
                        playlist_return_type
                            .playlists
                            .iter_mut()
                            .for_each(|p| sanitize_playlist(&prefix, p));
                    }
                    ExtensionExtraEventResponse::RequestedPlaylistSongs(
                        songs_with_page_token_return_type,
                    ) => {
                        songs_with_page_token_return_type
                            .songs
                            .iter_mut()
                            .for_each(|s| sanitize_song(&prefix, s));
                    }
                    ExtensionExtraEventResponse::OauthCallback => {}
                    ExtensionExtraEventResponse::SongQueueChanged => {}
                    ExtensionExtraEventResponse::Seeked => {}
                    ExtensionExtraEventResponse::VolumeChanged => {}
                    ExtensionExtraEventResponse::PlayerStateChanged => {}
                    ExtensionExtraEventResponse::SongChanged => {}
                    ExtensionExtraEventResponse::PreferenceChanged => {}
                    ExtensionExtraEventResponse::PlaybackDetailsRequested(_) => {}
                    ExtensionExtraEventResponse::CustomRequest(_) => {}
                    ExtensionExtraEventResponse::RequestedSongFromURL(song_return_type) => {
                        if let Some(song) = song_return_type.song.as_mut() {
                            sanitize_song(&prefix, song);
                        }
                    }
                    ExtensionExtraEventResponse::RequestedPlaylistFromURL(
                        playlist_and_songs_return_type,
                    ) => {
                        if let Some(playlist) = playlist_and_songs_return_type.playlist.as_mut() {
                            sanitize_playlist(&prefix, playlist);
                        }

                        if let Some(songs) = playlist_and_songs_return_type.songs.as_mut() {
                            songs.iter_mut().for_each(|s| sanitize_song(&prefix, s));
                        }
                    }
                    ExtensionExtraEventResponse::RequestedSearchResult(search_return_type) => {
                        search_return_type
                            .songs
                            .iter_mut()
                            .for_each(|s| sanitize_song(&prefix, s));
                        search_return_type
                            .albums
                            .iter_mut()
                            .for_each(|s| sanitize_album(&prefix, s));
                        search_return_type
                            .artists
                            .iter_mut()
                            .for_each(|s| sanitize_artist(&prefix, s));
                        search_return_type
                            .playlists
                            .iter_mut()
                            .for_each(|s| sanitize_playlist(&prefix, s));
                    }
                    ExtensionExtraEventResponse::RequestedRecommendations(
                        recommendations_return_type,
                    ) => {
                        recommendations_return_type
                            .songs
                            .iter_mut()
                            .for_each(|s| sanitize_song(&prefix, s));
                    }
                    ExtensionExtraEventResponse::RequestedLyrics(_) => {}
                    ExtensionExtraEventResponse::RequestedArtistSongs(
                        songs_with_page_token_return_type,
                    ) => {
                        songs_with_page_token_return_type
                            .songs
                            .iter_mut()
                            .for_each(|s| sanitize_song(&prefix, s));
                    }
                    ExtensionExtraEventResponse::RequestedAlbumSongs(
                        songs_with_page_token_return_type,
                    ) => {
                        songs_with_page_token_return_type
                            .songs
                            .iter_mut()
                            .for_each(|s| sanitize_song(&prefix, s));
                    }
                    ExtensionExtraEventResponse::SongAdded => {}
                    ExtensionExtraEventResponse::SongRemoved => {}
                    ExtensionExtraEventResponse::PlaylistAdded => {}
                    ExtensionExtraEventResponse::PlaylistRemoved => {}
                    ExtensionExtraEventResponse::RequestedSongFromId(song_return_type) => {
                        if let Some(song) = song_return_type.song.as_mut() {
                            sanitize_song(&prefix, song);
                        }
                    }
                    ExtensionExtraEventResponse::GetRemoteURL(_) => {}
                    ExtensionExtraEventResponse::Scrobble => {}
                    ExtensionExtraEventResponse::RequestedSongContextMenu(
                        _context_menu_return_type,
                    ) => {}
                    ExtensionExtraEventResponse::RequestedPlaylistContextMenu(
                        _context_menu_return_type,
                    ) => {}
                    ExtensionExtraEventResponse::ContextMenuAction => {}
                }
            }
            ExtensionCommandResponse::Empty => {}
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase", tag = "type", content = "data")]
pub enum ExtensionCommand {
    GetProviderScopes(PackageNameArgs),
    GetAccounts(PackageNameArgs),
    PerformAccountLogin(AccountLoginArgs),
    ExtraExtensionEvent(Box<ExtensionExtraEventArgs>),
}

impl TryFrom<(&str, &Value)> for ExtensionCommand {
    type Error = MoosyncError;
    #[tracing::instrument(level = "debug", skip())]
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
    pub fn get_package_name(&self) -> String {
        match self {
            Self::GetProviderScopes(args) => args.package_name.clone(),
            Self::GetAccounts(args) => args.package_name.clone(),
            Self::PerformAccountLogin(args) => args.package_name.clone(),
            Self::ExtraExtensionEvent(args) => args.package_name.clone(),
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn to_plugin_call(&self) -> (&'static str, Vec<u8>) {
        match self {
            Self::GetProviderScopes(_) => ("get_provider_scopes_wrapper", vec![]),
            Self::GetAccounts(_) => ("get_accounts_wrapper", vec![]),
            Self::PerformAccountLogin(args) => (
                "perform_account_login_wrapper",
                serde_json::to_vec(&args).unwrap(),
            ),

            // TODO: Why the fuck did I decide to split some events as "extra"
            Self::ExtraExtensionEvent(args) => {
                let res = match &args.data {
                    ExtensionExtraEvent::RequestedPlaylists(_) => ("get_playlists_wrapper", vec![]),
                    ExtensionExtraEvent::RequestedPlaylistSongs(id, _, token) => (
                        "get_playlist_content_wrapper",
                        serde_json::to_vec(&(id, token)).unwrap(),
                    ),
                    ExtensionExtraEvent::OauthCallback(code) => (
                        "oauth_callback_wrapper",
                        serde_json::to_vec(&code[0].clone()).unwrap(),
                    ),
                    ExtensionExtraEvent::SongQueueChanged(value) => (
                        "on_queue_changed_wrapper",
                        serde_json::to_vec(&value[0].clone()).unwrap(),
                    ),
                    ExtensionExtraEvent::Seeked(time) => {
                        ("on_seeked_wrapper", serde_json::to_vec(&time[0]).unwrap())
                    }
                    ExtensionExtraEvent::VolumeChanged(_) => ("on_volume_changed_wrapper", vec![]),
                    ExtensionExtraEvent::PlayerStateChanged(player_state) => (
                        "on_player_state_changed_wrapper",
                        serde_json::to_vec(&player_state[0]).unwrap(),
                    ),
                    ExtensionExtraEvent::SongChanged(_) => ("on_song_changed_wrapper", vec![]),
                    ExtensionExtraEvent::PreferenceChanged(preferences) => (
                        "on_preferences_changed_wrapper",
                        serde_json::to_vec(&preferences[0].clone()).unwrap(),
                    ),
                    ExtensionExtraEvent::PlaybackDetailsRequested(songs) => (
                        "get_playback_details_wrapper",
                        serde_json::to_vec(&songs[0].clone()).unwrap(),
                    ),
                    ExtensionExtraEvent::CustomRequest(url) => (
                        "handle_custom_request_wrapper",
                        serde_json::to_vec(&url[0].clone()).unwrap(),
                    ),
                    ExtensionExtraEvent::RequestedSongFromURL(url, _) => (
                        "get_song_from_url_wrapper",
                        serde_json::to_vec(&url).unwrap(),
                    ),
                    ExtensionExtraEvent::RequestedPlaylistFromURL(url, _) => (
                        "get_playlist_from_url_wrapper",
                        serde_json::to_vec(&url).unwrap(),
                    ),
                    ExtensionExtraEvent::RequestedSearchResult(term) => (
                        "search_wrapper",
                        serde_json::to_vec(&term[0].clone()).unwrap(),
                    ),
                    ExtensionExtraEvent::RequestedRecommendations => {
                        ("get_recommendations_wrapper", vec![])
                    }
                    ExtensionExtraEvent::RequestedLyrics(song) => (
                        "get_lyrics_wrapper",
                        serde_json::to_vec(&song.clone()).unwrap(),
                    ),
                    ExtensionExtraEvent::RequestedArtistSongs(artist, token) => (
                        "get_artist_songs_wrapper",
                        serde_json::to_vec(&(artist.clone(), token)).unwrap(),
                    ),
                    ExtensionExtraEvent::RequestedAlbumSongs(album, token) => (
                        "get_album_songs_wrapper",
                        serde_json::to_vec(&(album.clone(), token)).unwrap(),
                    ),
                    ExtensionExtraEvent::SongAdded(song) => (
                        "on_song_added_wrapper",
                        serde_json::to_vec(&song[0].clone()).unwrap(),
                    ),
                    ExtensionExtraEvent::SongRemoved(song) => (
                        "on_song_removed_wrapper",
                        serde_json::to_vec(&song[0].clone()).unwrap(),
                    ),
                    ExtensionExtraEvent::PlaylistAdded(playlist) => (
                        "on_playlist_added_wrapper",
                        serde_json::to_vec(&playlist[0].clone()).unwrap(),
                    ),
                    ExtensionExtraEvent::PlaylistRemoved(playlist) => (
                        "on_playlist_removed_wrapper",
                        serde_json::to_vec(&playlist[0].clone()).unwrap(),
                    ),
                    ExtensionExtraEvent::RequestedSongFromId(id) => (
                        "get_song_from_id_wrapper",
                        serde_json::to_vec(&id[0].clone()).unwrap(),
                    ),
                    ExtensionExtraEvent::GetRemoteURL(_) => ("get_remote_url_wrapper", vec![]),
                    ExtensionExtraEvent::Scrobble(song) => (
                        "scrobble_wrapper",
                        serde_json::to_vec(&song[0].clone()).unwrap(),
                    ),
                    ExtensionExtraEvent::RequestedSongContextMenu(song) => (
                        "get_song_context_menu_wrapper",
                        serde_json::to_vec(&song[0].clone()).unwrap(),
                    ),
                    ExtensionExtraEvent::RequestedPlaylistContextMenu(playlist) => (
                        "get_playlist_context_menu_wrapper",
                        serde_json::to_vec(&playlist[0].clone()).unwrap(),
                    ),
                    ExtensionExtraEvent::ContextMenuAction(action_id) => (
                        "on_context_menu_action_wrapper",
                        serde_json::to_vec(&action_id[0].clone()).unwrap(),
                    ),
                };
                (res.0, res.1)
            }
        }
    }

    #[tracing::instrument(level = "debug", skip(self, value))]
    pub fn parse_response(&self, value: Value) -> MoosyncResult<ExtensionCommandResponse> {
        let ret = match self {
            Self::GetProviderScopes(_) => {
                ExtensionCommandResponse::GetProviderScopes(serde_json::from_value(value)?)
            }
            Self::GetAccounts(_) => {
                ExtensionCommandResponse::GetAccounts(serde_json::from_value(value)?)
            }
            Self::PerformAccountLogin(_) => {
                ExtensionCommandResponse::PerformAccountLogin(serde_json::from_value(value)?)
            }
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
                    ExtensionExtraEvent::RequestedSongContextMenu(_) => {
                        ExtensionExtraEventResponse::RequestedSongContextMenu(
                            serde_json::from_value(value)?,
                        )
                    }
                    ExtensionExtraEvent::RequestedPlaylistContextMenu(_) => {
                        ExtensionExtraEventResponse::RequestedPlaylistContextMenu(
                            serde_json::from_value(value)?,
                        )
                    }
                    ExtensionExtraEvent::ContextMenuAction(_) => {
                        ExtensionExtraEventResponse::ContextMenuAction
                    }
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
    GetDisplayName(PackageNameArgs),
}

impl TryFrom<(&str, &Value)> for RunnerCommand {
    type Error = MoosyncError;

    #[tracing::instrument(level = "debug", skip())]
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
            "getDisplayName" => Ok(Self::GetDisplayName(
                serde_json::from_value(data.clone()).unwrap(),
            )),
            _ => Err("Failed to parse runner command".into()),
        }
    }
}

#[derive(Debug)]
pub enum RunnerCommandResp {
    ExtensionList(Vec<types::ui::extensions::ExtensionDetail>),
    ExtensionIcon(Option<String>),
    ExtensionName(Option<String>),
    Empty(),
}
