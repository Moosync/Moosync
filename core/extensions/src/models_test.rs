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

use assertables::{assert_none, assert_ok, assert_some_eq_x};
use extensions_proto::moosync::types::{
    AddPlaylistRequest, AddSongsRequest, AddToPlaylistRequest, ExtensionAccountDetail,
    GetPlayerStateRequest, GetPreferenceRequest, GetSecureRequest, MainCommand, PreferenceData,
    RemoveSongRequest, SetAccountRequest, SetPreferenceRequest, SetSecureRequest,
    UpdateSongRequest, main_command,
};
use songs_proto::moosync::types::{InnerSong, Playlist, Song};
use tracing_test::traced_test;

use crate::models::SanitizeCommand;

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_models_sanitize_add_songs() {
    let song = Song {
        song: Some(InnerSong {
            id: Some("123".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let mut command = MainCommand {
        command: Some(main_command::Command::AddSongs(AddSongsRequest {
            songs: vec![song],
        })),
    };

    assert_ok!(command.sanitize("my.pkg"));
    let Some(main_command::Command::AddSongs(req)) = command.command else {
        panic!("Expected AddSongs command");
    };
    assert_some_eq_x!(
        req.songs[0].song.as_ref().unwrap().id.as_deref(),
        "my.pkg:123"
    );
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_models_sanitize_playlist_and_update() {
    let mut command = MainCommand {
        command: Some(main_command::Command::AddPlaylist(AddPlaylistRequest {
            playlist: Some(Playlist {
                playlist_id: Some("pl_id".to_string()),
                ..Default::default()
            }),
        })),
    };

    assert_ok!(command.sanitize("my.pkg"));
    let Some(main_command::Command::AddPlaylist(req)) = command.command else {
        panic!("Expected AddPlaylist command");
    };
    assert_some_eq_x!(
        req.playlist.as_ref().unwrap().playlist_id.as_deref(),
        "my.pkg:pl_id"
    );
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_models_sanitize_preferences() {
    let mut get_preference = MainCommand {
        command: Some(main_command::Command::GetPreference(GetPreferenceRequest {
            data: Some(PreferenceData {
                key: "user_key".to_string(),
                value: None,
            }),
        })),
    };
    let mut set_preference = MainCommand {
        command: Some(main_command::Command::SetPreference(SetPreferenceRequest {
            data: Some(PreferenceData {
                key: "theme".to_string(),
                value: None,
            }),
        })),
    };
    let mut get_secure = MainCommand {
        command: Some(main_command::Command::GetSecure(GetSecureRequest {
            data: Some(PreferenceData {
                key: "token".to_string(),
                value: None,
            }),
        })),
    };
    let mut set_secure = MainCommand {
        command: Some(main_command::Command::SetSecure(SetSecureRequest {
            data: Some(PreferenceData {
                key: "secret".to_string(),
                value: None,
            }),
        })),
    };
    let mut empty_data = MainCommand {
        command: Some(main_command::Command::GetPreference(GetPreferenceRequest {
            data: None,
        })),
    };

    assert_ok!(get_preference.sanitize("pkg.a"));
    assert_ok!(set_preference.sanitize("pkg.b"));
    assert_ok!(get_secure.sanitize("pkg.c"));
    assert_ok!(set_secure.sanitize("pkg.d"));
    assert_ok!(empty_data.sanitize("pkg.e"));

    let Some(main_command::Command::GetPreference(get_pref_req)) = get_preference.command else {
        panic!("Expected GetPreference");
    };
    assert_eq!(get_pref_req.data.unwrap().key, "pkg.a.user_key");

    let Some(main_command::Command::SetPreference(set_pref_req)) = set_preference.command else {
        panic!("Expected SetPreference");
    };
    assert_eq!(set_pref_req.data.unwrap().key, "pkg.b.theme");

    let Some(main_command::Command::GetSecure(get_sec_req)) = get_secure.command else {
        panic!("Expected GetSecure");
    };
    assert_eq!(get_sec_req.data.unwrap().key, "pkg.c.token");

    let Some(main_command::Command::SetSecure(set_sec_req)) = set_secure.command else {
        panic!("Expected SetSecure");
    };
    assert_eq!(set_sec_req.data.unwrap().key, "pkg.d.secret");

    let Some(main_command::Command::GetPreference(empty_req)) = empty_data.command else {
        panic!("Expected GetPreference with empty data");
    };
    assert_none!(&empty_req.data);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_models_sanitize_songs_variations() {
    let mut remove_song = MainCommand {
        command: Some(main_command::Command::RemoveSong(RemoveSongRequest {
            song: Some(Song {
                song: Some(InnerSong {
                    id: Some("song_1".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        })),
    };
    let mut remove_empty = MainCommand {
        command: Some(main_command::Command::RemoveSong(RemoveSongRequest {
            song: None,
        })),
    };
    let mut update_song = MainCommand {
        command: Some(main_command::Command::UpdateSong(UpdateSongRequest {
            song: Some(Song {
                song: Some(InnerSong {
                    id: Some("song_2".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        })),
    };
    let mut update_empty = MainCommand {
        command: Some(main_command::Command::UpdateSong(UpdateSongRequest {
            song: None,
        })),
    };
    let mut add_to_playlist = MainCommand {
        command: Some(main_command::Command::AddToPlaylist(AddToPlaylistRequest {
            playlist_id: "pl_1".to_string(),
            songs: vec![Song {
                song: Some(InnerSong {
                    id: Some("song_3".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }],
        })),
    };

    assert_ok!(remove_song.sanitize("my_pkg"));
    assert_ok!(remove_empty.sanitize("my_pkg"));
    assert_ok!(update_song.sanitize("my_pkg"));
    assert_ok!(update_empty.sanitize("my_pkg"));
    assert_ok!(add_to_playlist.sanitize("my_pkg"));

    let Some(main_command::Command::RemoveSong(rem_req)) = remove_song.command else {
        panic!("Expected RemoveSong");
    };
    assert_eq!(
        rem_req.song.unwrap().song.unwrap().id.as_deref(),
        Some("my_pkg:song_1")
    );

    let Some(main_command::Command::UpdateSong(upd_req)) = update_song.command else {
        panic!("Expected UpdateSong");
    };
    assert_eq!(
        upd_req.song.unwrap().song.unwrap().id.as_deref(),
        Some("my_pkg:song_2")
    );

    let Some(main_command::Command::AddToPlaylist(pl_req)) = add_to_playlist.command else {
        panic!("Expected AddToPlaylist");
    };
    assert_eq!(
        pl_req.songs[0].song.as_ref().unwrap().id.as_deref(),
        Some("my_pkg:song_3")
    );
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_models_sanitize_entity_and_edge_cases() {
    let mut add_empty_playlist = MainCommand {
        command: Some(main_command::Command::AddPlaylist(AddPlaylistRequest {
            playlist: None,
        })),
    };
    let mut set_account = MainCommand {
        command: Some(main_command::Command::SetAccount(SetAccountRequest {
            account: Some(ExtensionAccountDetail {
                id: "acc_1".to_string(),
                name: "Account".to_string(),
                package_name: "".to_string(),
                logged_in: true,
                ..Default::default()
            }),
        })),
    };
    let mut set_account_empty = MainCommand {
        command: Some(main_command::Command::SetAccount(SetAccountRequest {
            account: None,
        })),
    };
    let mut unhandled_command = MainCommand {
        command: Some(main_command::Command::GetPlayerState(
            GetPlayerStateRequest {},
        )),
    };
    let mut none_command = MainCommand { command: None };

    assert_ok!(add_empty_playlist.sanitize("test_pkg"));
    assert_ok!(set_account.sanitize("test_pkg"));
    assert_ok!(set_account_empty.sanitize("test_pkg"));
    assert_ok!(unhandled_command.sanitize("test_pkg"));
    assert_ok!(none_command.sanitize("test_pkg"));

    let Some(main_command::Command::SetAccount(acc_req)) = set_account.command else {
        panic!("Expected SetAccount");
    };
    assert_eq!(acc_req.account.unwrap().package_name, "test_pkg");
    assert_none!(&none_command.command);
}
