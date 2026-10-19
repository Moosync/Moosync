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

use assertables::{assert_err, assert_none, assert_ok};
use extensions_proto::moosync::types::{
    AddPlaylistRequest, AddSongsRequest, AddToPlaylistRequest, ExtensionAccountDetail,
    ExtensionsUpdatedRequest, GetAppVersionRequest, GetCurrentSongRequest, GetEntityRequest,
    GetPlayerStateRequest, GetPreferenceRequest, GetQueueRequest, GetSecureRequest, GetSongRequest,
    GetTimeRequest, GetVolumeRequest, OpenExternalUrlRequest, PreferenceData, RegisterOauthRequest,
    RegisterUserPreferenceRequest, RemoveSongRequest, SetAccountRequest, SetPreferenceRequest,
    SetSecureRequest, UnregisterUserPreferenceRequest, UpdateSongRequest, main_command,
    main_command_response,
};
use preferences_proto::moosync::types::PreferenceItem;
use songs_proto::moosync::types::{EntityResult, GetEntityOptions, GetSongOptions, Playlist, Song};
use tracing_test::traced_test;

use crate::{
    context::{DispatchCommand, ReplyHandler},
    errors::ExtensionError,
};

struct MockReply;
impl ReplyHandler for MockReply {
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_song(&self, _: &str, _: GetSongOptions) -> Result<Vec<Song>, ExtensionError> {
        Ok(vec![Song::default()])
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_entity(&self, _: &str, _: GetEntityOptions) -> Result<EntityResult, ExtensionError> {
        Ok(EntityResult::default())
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_current_song(&self, _: &str) -> Result<Option<Song>, ExtensionError> {
        Ok(Some(Song::default()))
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_player_state(&self, _: &str) -> Result<i32, ExtensionError> { Ok(1) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_volume(&self, _: &str) -> Result<f64, ExtensionError> { Ok(0.8) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_time(&self, _: &str) -> Result<f64, ExtensionError> { Ok(42.0) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_queue(&self, _: &str) -> Result<(Vec<Song>, usize), ExtensionError> {
        Ok((vec![Song::default()], 2))
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_preference(
        &self,
        _: &str,
        _: &str,
    ) -> Result<Option<extensions_proto::struct_proto::google::protobuf::Value>, ExtensionError>
    {
        Ok(None)
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn set_preference(
        &self,
        _: &str,
        _: &str,
        _: extensions_proto::struct_proto::google::protobuf::Value,
    ) -> Result<bool, ExtensionError> {
        Ok(true)
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_secure(
        &self,
        _: &str,
        _: &str,
    ) -> Result<Option<extensions_proto::struct_proto::google::protobuf::Value>, ExtensionError>
    {
        Ok(None)
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn set_secure(
        &self,
        _: &str,
        _: &str,
        _: extensions_proto::struct_proto::google::protobuf::Value,
    ) -> Result<bool, ExtensionError> {
        Ok(true)
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn add_songs(&self, _: &str, _: Vec<Song>) -> Result<Vec<Song>, ExtensionError> {
        Ok(vec![Song::default()])
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn remove_song(&self, _: &str, _: Song) -> Result<bool, ExtensionError> { Ok(true) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn update_song(&self, _: &str, s: Song) -> Result<Song, ExtensionError> { Ok(s) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn add_playlist(&self, _: &str, _: Playlist) -> Result<String, ExtensionError> {
        Ok("pl1".to_string())
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn add_to_playlist(&self, _: &str, _: String, _: Vec<Song>) -> Result<bool, ExtensionError> {
        Ok(true)
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn register_oauth(&self, _: &str, _: String) -> Result<bool, ExtensionError> { Ok(true) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn open_external_url(&self, _: &str, _: String) -> Result<bool, ExtensionError> { Ok(true) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn set_account(&self, _: &str, _: ExtensionAccountDetail) -> Result<bool, ExtensionError> {
        Ok(true)
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn register_user_preference(
        &self,
        _: &str,
        _: Vec<PreferenceItem>,
    ) -> Result<bool, ExtensionError> {
        Ok(true)
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn unregister_user_preference(&self, _: &str, _: Vec<String>) -> Result<bool, ExtensionError> {
        Ok(true)
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn extensions_updated(&self, _: &str) -> Result<(), ExtensionError> { Ok(()) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_app_version(&self, _: &str) -> Result<String, ExtensionError> { Ok("2.0.0".to_string()) }
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_dispatch_command_handlers() {
    let mock = MockReply;

    let cmd_state = main_command::Command::GetPlayerState(GetPlayerStateRequest {});
    let res_state = cmd_state.dispatch(&mock, "pkg");
    let cmd_vol = main_command::Command::GetVolume(GetVolumeRequest {});
    let res_vol = cmd_vol.dispatch(&mock, "pkg");
    let cmd_time = main_command::Command::GetTime(GetTimeRequest {});
    let res_time = cmd_time.dispatch(&mock, "pkg");
    let cmd_ver = main_command::Command::GetAppVersion(GetAppVersionRequest {});
    let res_ver = cmd_ver.dispatch(&mock, "pkg");

    assert_ok!(res_state.as_ref());
    assert_ok!(res_vol.as_ref());
    assert_ok!(res_time.as_ref());
    assert_ok!(res_ver.as_ref());

    match res_state.unwrap() {
        main_command_response::Response::GetPlayerState(r) => assert_eq!(r.state, 1),
        _ => panic!("Expected GetPlayerState"),
    }
    match res_vol.unwrap() {
        main_command_response::Response::GetVolume(r) => assert_eq!(r.volume, 0.8),
        _ => panic!("Expected GetVolume"),
    }
    match res_time.unwrap() {
        main_command_response::Response::GetTime(r) => assert_eq!(r.time, 42.0),
        _ => panic!("Expected GetTime"),
    }
    match res_ver.unwrap() {
        main_command_response::Response::GetAppVersion(r) => assert_eq!(r.version, "2.0.0"),
        _ => panic!("Expected GetAppVersion"),
    }
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_dispatch_song_and_queue_commands() {
    let mock = MockReply;

    let cmd_song = main_command::Command::GetSong(GetSongRequest { options: None });
    let res_song = cmd_song.dispatch(&mock, "pkg");
    let cmd_entity = main_command::Command::GetEntity(GetEntityRequest { options: None });
    let res_entity = cmd_entity.dispatch(&mock, "pkg");
    let cmd_curr = main_command::Command::GetCurrentSong(GetCurrentSongRequest {});
    let res_curr = cmd_curr.dispatch(&mock, "pkg");
    let cmd_queue = main_command::Command::GetQueue(GetQueueRequest {});
    let res_queue = cmd_queue.dispatch(&mock, "pkg");

    assert_ok!(res_song.as_ref());
    assert_ok!(res_entity.as_ref());
    assert_ok!(res_curr.as_ref());
    assert_ok!(res_queue.as_ref());

    match res_song.unwrap() {
        main_command_response::Response::GetSong(r) => assert_eq!(r.songs.len(), 1),
        _ => panic!("Expected GetSong"),
    }
    match res_entity.unwrap() {
        main_command_response::Response::GetEntity(r) => assert!(r.entity.is_some()),
        _ => panic!("Expected GetEntity"),
    }
    match res_curr.unwrap() {
        main_command_response::Response::GetCurrentSong(r) => assert!(r.song.is_some()),
        _ => panic!("Expected GetCurrentSong"),
    }
    match res_queue.unwrap() {
        main_command_response::Response::GetQueue(r) => {
            let q = r.queue.unwrap();
            assert_eq!(q.songs.len(), 1);
            assert_eq!(q.index, 2);
        }
        _ => panic!("Expected GetQueue"),
    }
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_dispatch_preference_and_secure_commands() {
    let mock = MockReply;

    let cmd_get_pref = main_command::Command::GetPreference(GetPreferenceRequest {
        data: Some(PreferenceData {
            key: "k1".to_string(),
            value: None,
        }),
    });
    let res_get_pref = cmd_get_pref.dispatch(&mock, "pkg");

    let cmd_set_pref = main_command::Command::SetPreference(SetPreferenceRequest {
        data: Some(PreferenceData {
            key: "k2".to_string(),
            value: None,
        }),
    });
    let res_set_pref = cmd_set_pref.dispatch(&mock, "pkg");

    let cmd_set_pref_none =
        main_command::Command::SetPreference(SetPreferenceRequest { data: None });
    let res_set_pref_none = cmd_set_pref_none.dispatch(&mock, "pkg");

    let cmd_get_sec = main_command::Command::GetSecure(GetSecureRequest {
        data: Some(PreferenceData {
            key: "k3".to_string(),
            value: None,
        }),
    });
    let res_get_sec = cmd_get_sec.dispatch(&mock, "pkg");

    let cmd_set_sec = main_command::Command::SetSecure(SetSecureRequest {
        data: Some(PreferenceData {
            key: "k4".to_string(),
            value: None,
        }),
    });
    let res_set_sec = cmd_set_sec.dispatch(&mock, "pkg");

    let cmd_set_sec_none = main_command::Command::SetSecure(SetSecureRequest { data: None });
    let res_set_sec_none = cmd_set_sec_none.dispatch(&mock, "pkg");

    let cmd_reg_pref =
        main_command::Command::RegisterUserPreference(RegisterUserPreferenceRequest {
            prefs: vec![],
        });
    let res_reg_pref = cmd_reg_pref.dispatch(&mock, "pkg");

    let cmd_unreg_pref =
        main_command::Command::UnregisterUserPreference(UnregisterUserPreferenceRequest {
            keys: vec![],
        });
    let res_unreg_pref = cmd_unreg_pref.dispatch(&mock, "pkg");

    assert_ok!(res_get_pref.as_ref());
    assert_ok!(res_set_pref.as_ref());
    assert_ok!(res_set_pref_none.as_ref());
    assert_ok!(res_get_sec.as_ref());
    assert_ok!(res_set_sec.as_ref());
    assert_ok!(res_set_sec_none.as_ref());
    assert_ok!(res_reg_pref.as_ref());
    assert_ok!(res_unreg_pref.as_ref());

    match res_set_pref.unwrap() {
        main_command_response::Response::SetPreference(r) => assert!(r.success),
        _ => panic!("Expected SetPreference"),
    }
    match res_set_pref_none.unwrap() {
        main_command_response::Response::SetPreference(r) => assert!(!r.success),
        _ => panic!("Expected SetPreference false"),
    }
    match res_set_sec.unwrap() {
        main_command_response::Response::SetSecure(r) => assert!(r.success),
        _ => panic!("Expected SetSecure"),
    }
    match res_set_sec_none.unwrap() {
        main_command_response::Response::SetSecure(r) => assert!(!r.success),
        _ => panic!("Expected SetSecure false"),
    }
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_dispatch_song_mutations_and_playlists() {
    let mock = MockReply;

    let cmd_add_songs = main_command::Command::AddSongs(AddSongsRequest { songs: vec![] });
    let res_add_songs = cmd_add_songs.dispatch(&mock, "pkg");

    let cmd_rem_song = main_command::Command::RemoveSong(RemoveSongRequest {
        song: Some(Song::default()),
    });
    let res_rem_song = cmd_rem_song.dispatch(&mock, "pkg");

    let cmd_rem_song_none = main_command::Command::RemoveSong(RemoveSongRequest { song: None });
    let res_rem_song_none = cmd_rem_song_none.dispatch(&mock, "pkg");

    let cmd_upd_song = main_command::Command::UpdateSong(UpdateSongRequest {
        song: Some(Song::default()),
    });
    let res_upd_song = cmd_upd_song.dispatch(&mock, "pkg");

    let cmd_upd_song_err = main_command::Command::UpdateSong(UpdateSongRequest { song: None });
    let res_upd_song_err = cmd_upd_song_err.dispatch(&mock, "pkg");

    let cmd_add_pl = main_command::Command::AddPlaylist(AddPlaylistRequest {
        playlist: Some(Playlist::default()),
    });
    let res_add_pl = cmd_add_pl.dispatch(&mock, "pkg");

    let cmd_add_pl_err = main_command::Command::AddPlaylist(AddPlaylistRequest { playlist: None });
    let res_add_pl_err = cmd_add_pl_err.dispatch(&mock, "pkg");

    let cmd_add_to_pl = main_command::Command::AddToPlaylist(AddToPlaylistRequest {
        playlist_id: "p1".to_string(),
        songs: vec![],
    });
    let res_add_to_pl = cmd_add_to_pl.dispatch(&mock, "pkg");

    assert_ok!(res_add_songs.as_ref());
    assert_ok!(res_rem_song.as_ref());
    assert_ok!(res_rem_song_none.as_ref());
    assert_ok!(res_upd_song.as_ref());
    assert_err!(res_upd_song_err.as_ref());
    assert_ok!(res_add_pl.as_ref());
    assert_err!(res_add_pl_err.as_ref());
    assert_ok!(res_add_to_pl.as_ref());

    match res_add_pl.unwrap() {
        main_command_response::Response::AddPlaylist(r) => assert_eq!(r.playlist_id, "pl1"),
        _ => panic!("Expected AddPlaylist"),
    }
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_dispatch_oauth_account_and_misc() {
    let mock = MockReply;

    let cmd_oauth = main_command::Command::RegisterOauth(RegisterOauthRequest {
        url: "http://oauth".to_string(),
    });
    let res_oauth = cmd_oauth.dispatch(&mock, "pkg");

    let cmd_open = main_command::Command::OpenExternalUrl(OpenExternalUrlRequest {
        url: "http://open".to_string(),
    });
    let res_open = cmd_open.dispatch(&mock, "pkg");

    let cmd_acc = main_command::Command::SetAccount(SetAccountRequest {
        account: Some(ExtensionAccountDetail::default()),
    });
    let res_acc = cmd_acc.dispatch(&mock, "pkg");

    let cmd_acc_err = main_command::Command::SetAccount(SetAccountRequest { account: None });
    let res_acc_err = cmd_acc_err.dispatch(&mock, "pkg");

    let cmd_updated = main_command::Command::ExtensionsUpdated(ExtensionsUpdatedRequest::default());
    let res_updated = cmd_updated.dispatch(&mock, "pkg");

    assert_ok!(res_oauth.as_ref());
    assert_ok!(res_open.as_ref());
    assert_ok!(res_acc.as_ref());
    assert_err!(res_acc_err.as_ref());
    assert_ok!(res_updated.as_ref());
    assert_none!(mock.get_runtime_handle());
}
