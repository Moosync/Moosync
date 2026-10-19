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

use assertables::{assert_is_empty, assert_len_eq_x, assert_none, assert_some_eq_x};
use extensions_proto::moosync::types::{
    ExtensionCommandResponse, RequestedAlbumSongsResponse, RequestedArtistSongsResponse,
    RequestedPlaylistFromUrlResponse, RequestedPlaylistSongsResponse, RequestedPlaylistsResponse,
    RequestedRecommendationsResponse, RequestedSearchResultResponse, RequestedSongFromIdResponse,
    RequestedSongFromUrlResponse, extension_command_response,
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
fn test_sanitize_song_from_url_and_id() {
    let from_url = ExtensionCommandResponse {
        response: Some(extension_command_response::Response::RequestedSongFromUrl(
            RequestedSongFromUrlResponse {
                song: Some(create_test_song("url_song")),
            },
        )),
    };
    let from_url_empty = ExtensionCommandResponse {
        response: Some(extension_command_response::Response::RequestedSongFromUrl(
            RequestedSongFromUrlResponse { song: None },
        )),
    };
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

    let sanitized_url = from_url.sanitize("my_ext");
    let sanitized_url_empty = from_url_empty.sanitize("my_ext");
    let sanitized_id = from_id.sanitize("my_ext");
    let sanitized_id_empty = from_id_empty.sanitize("my_ext");

    let Some(extension_command_response::Response::RequestedSongFromUrl(r1)) =
        sanitized_url.response
    else {
        panic!("Expected RequestedSongFromUrl");
    };
    assert_some_eq_x!(
        r1.song
            .as_ref()
            .unwrap()
            .song
            .as_ref()
            .unwrap()
            .id
            .as_deref(),
        "my_ext:url_song"
    );

    let Some(extension_command_response::Response::RequestedSongFromUrl(r2)) =
        sanitized_url_empty.response
    else {
        panic!("Expected RequestedSongFromUrl empty");
    };
    assert_none!(&r2.song);

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
fn test_sanitize_playlist_from_url() {
    let response = ExtensionCommandResponse {
        response: Some(
            extension_command_response::Response::RequestedPlaylistFromUrl(
                RequestedPlaylistFromUrlResponse {
                    playlist: Some(create_test_playlist("pl_url")),
                    songs: vec![create_test_song("pl_song")],
                },
            ),
        ),
    };
    let empty_response = ExtensionCommandResponse {
        response: Some(
            extension_command_response::Response::RequestedPlaylistFromUrl(
                RequestedPlaylistFromUrlResponse {
                    playlist: None,
                    songs: vec![],
                },
            ),
        ),
    };

    let sanitized = response.sanitize("plug");
    let sanitized_empty = empty_response.sanitize("plug");

    let Some(extension_command_response::Response::RequestedPlaylistFromUrl(r1)) =
        sanitized.response
    else {
        panic!("Expected RequestedPlaylistFromUrl");
    };
    assert_some_eq_x!(
        r1.playlist.as_ref().unwrap().playlist_id.as_deref(),
        "plug:pl_url"
    );
    assert_some_eq_x!(
        r1.songs[0].song.as_ref().unwrap().id.as_deref(),
        "plug:pl_song"
    );

    let Some(extension_command_response::Response::RequestedPlaylistFromUrl(r2)) =
        sanitized_empty.response
    else {
        panic!("Expected RequestedPlaylistFromUrl empty");
    };
    assert_none!(&r2.playlist);
    assert_is_empty!(&r2.songs);
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
