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
    AddPlaylistRequest, AddSongsRequest, MainCommand, main_command,
};
use songs_proto::moosync::types::{InnerSong, Playlist, Song};

use crate::models::SanitizeCommand;

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_models_sanitize_add_songs() {
    let song = Song {
        song: Some(InnerSong {
            id: Some("123".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let mut cmd = MainCommand {
        command: Some(main_command::Command::AddSongs(AddSongsRequest {
            songs: vec![song],
        })),
    };

    cmd.sanitize("my.pkg").unwrap();

    if let Some(main_command::Command::AddSongs(req)) = cmd.command {
        assert_eq!(
            req.songs[0].song.as_ref().unwrap().id.as_deref(),
            Some("my.pkg:123")
        );
    } else {
        panic!("Expected AddSongs command");
    }
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_models_sanitize_playlist_and_update() {
    let mut cmd_pl = MainCommand {
        command: Some(main_command::Command::AddPlaylist(AddPlaylistRequest {
            playlist: Some(Playlist {
                playlist_id: Some("pl_id".to_string()),
                ..Default::default()
            }),
        })),
    };

    cmd_pl.sanitize("my.pkg").unwrap();

    if let Some(main_command::Command::AddPlaylist(req)) = cmd_pl.command {
        assert_eq!(
            req.playlist.as_ref().unwrap().playlist_id.as_deref(),
            Some("my.pkg:pl_id")
        );
    } else {
        panic!("Expected AddPlaylist command");
    }
}
