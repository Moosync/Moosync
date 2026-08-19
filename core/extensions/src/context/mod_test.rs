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

use extensions_proto::moosync::types::{
    GetAppVersionRequest, GetPlayerStateRequest, GetTimeRequest, GetVolumeRequest, MainCommand,
    main_command, main_command_response,
};
use songs_proto::moosync::types::{EntityResult, GetEntityOptions, GetSongOptions, Song};
use ui_proto::moosync::types::PreferenceUiData;

use crate::{
    context::{DispatchCommand, ReplyHandler},
    errors::ExtensionError,
};

struct MockReply;
impl ReplyHandler for MockReply {
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_song(&self, _: &str, _: GetSongOptions) -> Result<Vec<Song>, ExtensionError> {
        Ok(vec![])
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_entity(&self, _: &str, _: GetEntityOptions) -> Result<EntityResult, ExtensionError> {
        Ok(EntityResult::default())
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_current_song(&self, _: &str) -> Result<Option<Song>, ExtensionError> { Ok(None) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_player_state(&self, _: &str) -> Result<i32, ExtensionError> { Ok(1) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_volume(&self, _: &str) -> Result<f64, ExtensionError> { Ok(0.8) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_time(&self, _: &str) -> Result<f64, ExtensionError> { Ok(42.0) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_queue(&self, _: &str) -> Result<(Vec<Song>, usize), ExtensionError> { Ok((vec![], 0)) }
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
    fn add_songs(&self, _: &str, _: Vec<Song>) -> Result<Vec<Song>, ExtensionError> { Ok(vec![]) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn remove_song(&self, _: &str, _: Song) -> Result<bool, ExtensionError> { Ok(true) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn update_song(&self, _: &str, s: Song) -> Result<Song, ExtensionError> { Ok(s) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn add_playlist(
        &self,
        _: &str,
        _: songs_proto::moosync::types::Playlist,
    ) -> Result<String, ExtensionError> {
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
    fn update_accounts(&self, _: &str, _: Option<String>) -> Result<bool, ExtensionError> {
        Ok(true)
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn register_user_preference(
        &self,
        _: &str,
        _: Vec<PreferenceUiData>,
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
#[tracing::instrument(level = "debug", skip_all)]
fn test_dispatch_command_handlers() {
    let mock = MockReply;

    let cmd_state = main_command::Command::GetPlayerState(GetPlayerStateRequest {});
    let res = cmd_state.dispatch(&mock, "pkg").unwrap();
    match res {
        main_command_response::Response::GetPlayerState(r) => assert_eq!(r.state, 1),
        _ => panic!("Expected GetPlayerState"),
    }

    let cmd_vol = main_command::Command::GetVolume(GetVolumeRequest {});
    let res = cmd_vol.dispatch(&mock, "pkg").unwrap();
    match res {
        main_command_response::Response::GetVolume(r) => assert_eq!(r.volume, 0.8),
        _ => panic!("Expected GetVolume"),
    }

    let cmd_time = main_command::Command::GetTime(GetTimeRequest {});
    let res = cmd_time.dispatch(&mock, "pkg").unwrap();
    match res {
        main_command_response::Response::GetTime(r) => assert_eq!(r.time, 42.0),
        _ => panic!("Expected GetTime"),
    }

    let cmd_ver = main_command::Command::GetAppVersion(GetAppVersionRequest {});
    let res = cmd_ver.dispatch(&mock, "pkg").unwrap();
    match res {
        main_command_response::Response::GetAppVersion(r) => assert_eq!(r.version, "2.0.0"),
        _ => panic!("Expected GetAppVersion"),
    }
}
