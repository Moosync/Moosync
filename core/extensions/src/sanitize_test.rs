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

use assertables::{assert_len_eq_x, assert_none, assert_some_eq_x};
use extensions_proto::moosync::types::{
    ContextMenuActionRequest, ContextMenuReturnType, ExtensionCommand, ExtensionCommandResponse,
    RequestedAlbumSongsResponse, RequestedArtistSongsResponse,
    RequestedPlaylistContextMenuResponse, RequestedPlaylistSongsResponse,
    RequestedPlaylistsResponse, RequestedRecommendationsResponse, RequestedSearchResultResponse,
    RequestedSongContextMenuResponse, RequestedSongFromIdResponse, extension_command,
    extension_command_response,
};
use songs_proto::moosync::types::{Album, Artist, InnerSong, Playlist, Song};
use tracing_test::traced_test;

use crate::sanitize::{
    Sanitize, sanitize_account, sanitize_album, sanitize_artist, sanitize_playlist, sanitize_song,
};

#[tracing::instrument(level = "debug", skip_all)]
fn create_test_song(id: &str) -> Song {
    Song {
        song: Some(InnerSong {
            id: Some(id.to_string()),
            ..Default::default()
        }),
        ..Default::default()
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn create_test_playlist(id: &str) -> Playlist {
    Playlist {
        playlist_id: Some(id.to_string()),
        ..Default::default()
    }
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_sanitize_extension_command_response_empty() {
    let response = ExtensionCommandResponse { response: None };

    let sanitized = response.sanitize("test_ext");

    assert_none!(&sanitized.response);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_sanitize_requested_playlists() {
    let response = ExtensionCommandResponse {
        response: Some(extension_command_response::Response::RequestedPlaylists(
            RequestedPlaylistsResponse {
                playlists: vec![create_test_playlist("pl1"), create_test_playlist("pl2")],
            },
        )),
    };

    let sanitized = response.sanitize("test_ext");

    let Some(extension_command_response::Response::RequestedPlaylists(result)) = sanitized.response
    else {
        panic!("Expected RequestedPlaylists");
    };
    assert_len_eq_x!(&result.playlists, 2);
    assert_some_eq_x!(result.playlists[0].playlist_id.as_deref(), "test_ext:pl1");
    assert_some_eq_x!(result.playlists[0].extension.as_deref(), "test_ext");
    assert_some_eq_x!(result.playlists[1].playlist_id.as_deref(), "test_ext:pl2");
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_sanitize_songs_responses() {
    let playlist_songs = ExtensionCommandResponse {
        response: Some(
            extension_command_response::Response::RequestedPlaylistSongs(
                RequestedPlaylistSongsResponse {
                    songs: vec![create_test_song("s1")],
                    ..Default::default()
                },
            ),
        ),
    };
    let artist_songs = ExtensionCommandResponse {
        response: Some(extension_command_response::Response::RequestedArtistSongs(
            RequestedArtistSongsResponse {
                songs: vec![create_test_song("s2")],
                ..Default::default()
            },
        )),
    };
    let album_songs = ExtensionCommandResponse {
        response: Some(extension_command_response::Response::RequestedAlbumSongs(
            RequestedAlbumSongsResponse {
                songs: vec![create_test_song("s3")],
                ..Default::default()
            },
        )),
    };
    let recommendations = ExtensionCommandResponse {
        response: Some(
            extension_command_response::Response::RequestedRecommendations(
                RequestedRecommendationsResponse {
                    songs: vec![create_test_song("s4")],
                },
            ),
        ),
    };

    let sanitized_playlist = playlist_songs.sanitize("ext");
    let sanitized_artist = artist_songs.sanitize("ext");
    let sanitized_album = album_songs.sanitize("ext");
    let sanitized_recommendations = recommendations.sanitize("ext");

    let Some(extension_command_response::Response::RequestedPlaylistSongs(r1)) =
        sanitized_playlist.response
    else {
        panic!("Expected RequestedPlaylistSongs");
    };
    assert_some_eq_x!(r1.songs[0].song.as_ref().unwrap().id.as_deref(), "ext:s1");

    let Some(extension_command_response::Response::RequestedArtistSongs(r2)) =
        sanitized_artist.response
    else {
        panic!("Expected RequestedArtistSongs");
    };
    assert_some_eq_x!(r2.songs[0].song.as_ref().unwrap().id.as_deref(), "ext:s2");

    let Some(extension_command_response::Response::RequestedAlbumSongs(r3)) =
        sanitized_album.response
    else {
        panic!("Expected RequestedAlbumSongs");
    };
    assert_some_eq_x!(r3.songs[0].song.as_ref().unwrap().id.as_deref(), "ext:s3");

    let Some(extension_command_response::Response::RequestedRecommendations(r4)) =
        sanitized_recommendations.response
    else {
        panic!("Expected RequestedRecommendations");
    };
    assert_some_eq_x!(r4.songs[0].song.as_ref().unwrap().id.as_deref(), "ext:s4");
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_sanitize_song_from_id() {
    let from_id = ExtensionCommandResponse {
        response: Some(extension_command_response::Response::RequestedSongFromId(
            RequestedSongFromIdResponse {
                song: Some(create_test_song("id_song")),
            },
        )),
    };
    let from_id_empty = ExtensionCommandResponse {
        response: Some(extension_command_response::Response::RequestedSongFromId(
            RequestedSongFromIdResponse { song: None },
        )),
    };

    let sanitized_id = from_id.sanitize("my_ext");
    let sanitized_id_empty = from_id_empty.sanitize("my_ext");

    let Some(extension_command_response::Response::RequestedSongFromId(r3)) = sanitized_id.response
    else {
        panic!("Expected RequestedSongFromId");
    };
    assert_some_eq_x!(
        r3.song
            .as_ref()
            .unwrap()
            .song
            .as_ref()
            .unwrap()
            .id
            .as_deref(),
        "my_ext:id_song"
    );

    let Some(extension_command_response::Response::RequestedSongFromId(r4)) =
        sanitized_id_empty.response
    else {
        panic!("Expected RequestedSongFromId empty");
    };
    assert_none!(&r4.song);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_sanitize_search_result() {
    let response = ExtensionCommandResponse {
        response: Some(extension_command_response::Response::RequestedSearchResult(
            RequestedSearchResultResponse {
                songs: vec![create_test_song("search_s1")],
                playlists: vec![create_test_playlist("search_p1")],
                artists: vec![Artist {
                    artist_id: Some("art_1".to_string()),
                    ..Default::default()
                }],
                albums: vec![Album {
                    album_id: Some("alb_1".to_string()),
                    ..Default::default()
                }],
            },
        )),
    };

    let sanitized = response.sanitize("search_ext");

    let Some(extension_command_response::Response::RequestedSearchResult(r)) = sanitized.response
    else {
        panic!("Expected RequestedSearchResult");
    };
    assert_some_eq_x!(
        r.songs[0].song.as_ref().unwrap().id.as_deref(),
        "search_ext:search_s1"
    );
    assert_some_eq_x!(
        r.playlists[0].playlist_id.as_deref(),
        "search_ext:search_p1"
    );
    assert_some_eq_x!(r.artists[0].artist_id.as_deref(), "search_ext:art_1");
    assert_some_eq_x!(r.albums[0].album_id.as_deref(), "search_ext:alb_1");
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_sanitize_id_already_prefixed() {
    let mut song = Song {
        song: Some(InnerSong {
            id: Some("my_ext:existing_id".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    sanitize_song(&mut song, "my_ext");

    assert_some_eq_x!(
        song.song.as_ref().unwrap().id.as_deref(),
        "my_ext:existing_id"
    );
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_sanitize_entities_and_account() {
    let mut artist = Artist {
        artist_id: Some("a1".to_string()),
        ..Default::default()
    };
    let mut album = Album {
        album_id: Some("al1".to_string()),
        ..Default::default()
    };
    let mut playlist = Playlist {
        playlist_id: Some("p1".to_string()),
        ..Default::default()
    };
    let mut account = extensions_proto::moosync::types::ExtensionAccountDetail {
        id: "acc_1".to_string(),
        package_name: "".to_string(),
        ..Default::default()
    };

    sanitize_artist(&mut artist, "ext");
    sanitize_album(&mut album, "ext");
    sanitize_playlist(&mut playlist, "ext");
    sanitize_account(&mut account, "ext");

    assert_some_eq_x!(artist.artist_id.as_deref(), "ext:a1");
    assert_some_eq_x!(artist.extension.as_deref(), "ext");
    assert_some_eq_x!(album.album_id.as_deref(), "ext:al1");
    assert_some_eq_x!(album.extension.as_deref(), "ext");
    assert_some_eq_x!(playlist.playlist_id.as_deref(), "ext:p1");
    assert_some_eq_x!(playlist.extension.as_deref(), "ext");
    assert_eq!(account.package_name, "ext");
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_sanitize_requested_song_context_menu() {
    let response = ExtensionCommandResponse {
        response: Some(
            extension_command_response::Response::RequestedSongContextMenu(
                RequestedSongContextMenuResponse {
                    menu: Some(ContextMenuReturnType {
                        name: "Custom Action".to_string(),
                        icon: "icon.png".to_string(),
                        action_id: "action_1".to_string(),
                    }),
                },
            ),
        ),
    };

    let sanitized = response.sanitize("sample_pkg");

    let Some(extension_command_response::Response::RequestedSongContextMenu(r)) =
        sanitized.response
    else {
        panic!("Expected RequestedSongContextMenu");
    };
    let menu = r.menu.expect("Expected menu item");
    assert_eq!(menu.name, "Custom Action");
    assert_eq!(menu.action_id, "extension:sample_pkg:action_1");
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_sanitize_requested_playlist_context_menu() {
    let response = ExtensionCommandResponse {
        response: Some(
            extension_command_response::Response::RequestedPlaylistContextMenu(
                RequestedPlaylistContextMenuResponse {
                    menu: Some(ContextMenuReturnType {
                        name: "Playlist Action".to_string(),
                        icon: "".to_string(),
                        action_id: "action_2".to_string(),
                    }),
                },
            ),
        ),
    };

    let sanitized = response.sanitize("sample_pkg");

    let Some(extension_command_response::Response::RequestedPlaylistContextMenu(r)) =
        sanitized.response
    else {
        panic!("Expected RequestedPlaylistContextMenu");
    };
    let menu = r.menu.expect("Expected menu item");
    assert_eq!(menu.name, "Playlist Action");
    assert_eq!(menu.action_id, "extension:sample_pkg:action_2");
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_sanitize_context_menu_already_prefixed() {
    let response = ExtensionCommandResponse {
        response: Some(
            extension_command_response::Response::RequestedSongContextMenu(
                RequestedSongContextMenuResponse {
                    menu: Some(ContextMenuReturnType {
                        name: "Custom Action".to_string(),
                        icon: "".to_string(),
                        action_id: "extension:sample_pkg:action_1".to_string(),
                    }),
                },
            ),
        ),
    };

    let sanitized = response.sanitize("sample_pkg");

    let Some(extension_command_response::Response::RequestedSongContextMenu(r)) =
        sanitized.response
    else {
        panic!("Expected RequestedSongContextMenu");
    };
    let menu = r.menu.expect("Expected menu item");
    assert_eq!(menu.action_id, "extension:sample_pkg:action_1");
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_sanitize_context_menu_action_command() {
    let cmd = ExtensionCommand {
        package_name: "sample_pkg".to_string(),
        event: Some(extension_command::Event::ContextMenuAction(
            ContextMenuActionRequest {
                action_id: "extension:sample_pkg:action_1".to_string(),
            },
        )),
    };

    let sanitized = cmd.sanitize("sample_pkg");

    let Some(extension_command::Event::ContextMenuAction(req)) = sanitized.event else {
        panic!("Expected ContextMenuAction");
    };
    assert_eq!(req.action_id, "action_1");
}
