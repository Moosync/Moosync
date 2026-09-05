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

use std::{
    fs,
    path::PathBuf,
    sync::{Arc, atomic::AtomicBool},
};

use assertables::{assert_is_empty, assert_len_eq_x};
use extensions_proto::moosync::types::{
    ExtensionCommandResponse, RequestedSearchResultResponse, extension_command_response,
};
use rstest::{fixture, rstest};
use songs_proto::moosync::types::{
    Album, Artist, EntityResult, GetEntityOptions, GetSongOptions, InnerSong, Playlist, Song,
};
use tempdir::TempDir;
use tracing_test::traced_test;
use ui_proto::moosync::types::PreferenceUiData;

use crate::{
    ReplyHandler,
    errors::ExtensionError,
    extension::Extension,
    sanitize::{Sanitize, sanitize_album, sanitize_artist, sanitize_playlist, sanitize_song},
};

struct DummyReply;
impl ReplyHandler for DummyReply {
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
    fn get_player_state(&self, _: &str) -> Result<i32, ExtensionError> { Ok(0) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_volume(&self, _: &str) -> Result<f64, ExtensionError> { Ok(1.0) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_time(&self, _: &str) -> Result<f64, ExtensionError> { Ok(0.0) }
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
        Ok("".to_string())
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
    fn get_app_version(&self, _: &str) -> Result<String, ExtensionError> { Ok("1.0.0".to_string()) }
}

struct TestExtContext {
    pub _temp_dir: TempDir,
    pub manifest_path: PathBuf,
    pub cache_dir: PathBuf,
}

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn ext_context() -> TestExtContext {
    let temp_dir = TempDir::new("moosync_ext_unit").expect("failed to create temp dir");
    let manifest_json = r#"{
        "name": "unit.pkg",
        "displayName": "Unit Test Extension",
        "version": "1.0.0",
        "extensionEntry": "main.wasm",
        "moosyncExtension": true,
        "icon": "icon.png",
        "author": "Tester"
    }"#;
    let manifest_path = temp_dir.path().join("package.json");
    fs::write(&manifest_path, manifest_json).unwrap();
    let lock_data = serde_json::json!({
        "registry": "local",
        "disabled": true
    });
    fs::write(
        temp_dir.path().join("extension.lock"),
        serde_json::to_vec(&lock_data).unwrap(),
    )
    .unwrap();
    let cache_dir = temp_dir.path().join("cache");

    TestExtContext {
        _temp_dir: temp_dir,
        manifest_path,
        cache_dir,
    }
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_extension_preferences_and_active_state(ext_context: TestExtContext) {
    let TestExtContext {
        manifest_path,
        cache_dir,
        ..
    } = ext_context;
    let reply = Arc::new(DummyReply);
    let has_started = Arc::new(AtomicBool::new(false));

    let ext = Extension::new(&manifest_path, reply, cache_dir, has_started);
    assert!(ext.is_ok());
    let ext = ext.unwrap();

    assert_eq!(ext.get_package_name(), "unit.pkg");
    assert!(!ext.is_active());
    assert_eq!(ext.get_lock_data().registry, "local");

    ext.register_ui_preferences(vec![PreferenceUiData {
        key: "volume".to_string(),
        title: "Default Volume".to_string(),
        ..Default::default()
    }]);
    let details = ext.get_extension_detail();
    assert_len_eq_x!(&details.preferences, 1);
    assert_eq!(details.preferences[0].key, "volume");

    ext.unregister_ui_preferences(vec!["volume".to_string()]);
    let details_after = ext.get_extension_detail();
    assert_is_empty!(&details_after.preferences);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_extension_load_manifest_without_optional_fields() {
    let temp_dir = TempDir::new("moosync_ext_no_icon").expect("failed to create temp dir");
    let manifest_json = r#"{
        "name": "sample.ext",
        "displayName": "Sample Extension",
        "version": "0.1.0",
        "extensionEntry": "main.wasm",
        "moosyncExtension": true
    }"#;
    let manifest_path = temp_dir.path().join("package.json");
    fs::write(&manifest_path, manifest_json).unwrap();
    let lock_data = serde_json::json!({
        "registry": "dev",
        "disabled": true
    });
    fs::write(
        temp_dir.path().join("extension.lock"),
        serde_json::to_vec(&lock_data).unwrap(),
    )
    .unwrap();
    let cache_dir = temp_dir.path().join("cache");
    let reply = Arc::new(DummyReply);
    let has_started = Arc::new(AtomicBool::new(false));

    let ext = Extension::new(&manifest_path, reply, cache_dir, has_started);

    assert!(ext.is_ok());
    let ext = ext.unwrap();
    assert_eq!(ext.get_package_name(), "sample.ext");
    let detail = ext.get_extension_detail();
    assert_eq!(detail.name, "Sample Extension");
    assert_eq!(detail.version, "0.1.0");
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_sanitize_artist() {
    let mut artist = Artist {
        artist_id: Some("art-1".to_string()),
        artist_name: Some("Artist One".to_string()),
        extension: None,
        ..Default::default()
    };

    sanitize_artist(&mut artist, "sample.ext");

    assert_eq!(artist.extension, Some("sample.ext".to_string()));
    assert_eq!(artist.artist_id, Some("sample.ext:art-1".to_string()));
    assert_eq!(artist.artist_name, Some("Artist One".to_string()));
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_sanitize_album() {
    let mut album = Album {
        album_id: Some("alb-1".to_string()),
        album_name: Some("Album One".to_string()),
        extension: None,
        ..Default::default()
    };

    sanitize_album(&mut album, "sample.ext");

    assert_eq!(album.extension, Some("sample.ext".to_string()));
    assert_eq!(album.album_id, Some("sample.ext:alb-1".to_string()));
    assert_eq!(album.album_name, Some("Album One".to_string()));
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_sanitize_playlist() {
    let mut playlist = Playlist {
        playlist_id: Some("pl-1".to_string()),
        playlist_name: "Playlist One".to_string(),
        extension: None,
        ..Default::default()
    };

    sanitize_playlist(&mut playlist, "sample.ext");

    assert_eq!(playlist.extension, Some("sample.ext".to_string()));
    assert_eq!(playlist.playlist_id, Some("sample.ext:pl-1".to_string()));
    assert_eq!(playlist.playlist_name, "Playlist One");
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_sanitize_song_without_album_and_artists() {
    let mut song = Song {
        song: Some(InnerSong {
            id: Some("song-1".to_string()),
            title: Some("Song One".to_string()),
            extension: None,
            ..Default::default()
        }),
        album: None,
        artists: vec![],
        genre: vec![],
    };

    sanitize_song(&mut song, "sample.ext");

    assert_eq!(
        song.song.as_ref().unwrap().extension,
        Some("sample.ext".to_string())
    );
    assert_eq!(
        song.song.as_ref().unwrap().id,
        Some("sample.ext:song-1".to_string())
    );
    assert!(song.album.is_none());
    assert_is_empty!(&song.artists);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_sanitize_song_with_album_and_multiple_artists() {
    let mut song = Song {
        song: Some(InnerSong {
            id: Some("song-2".to_string()),
            title: Some("Song Two".to_string()),
            extension: Some("old.ext".to_string()),
            ..Default::default()
        }),
        album: Some(Album {
            album_id: Some("alb-2".to_string()),
            album_name: Some("Album Two".to_string()),
            extension: Some("old.ext".to_string()),
            ..Default::default()
        }),
        artists: vec![
            Artist {
                artist_id: Some("art-2a".to_string()),
                artist_name: Some("Artist 2A".to_string()),
                extension: None,
                ..Default::default()
            },
            Artist {
                artist_id: Some("art-2b".to_string()),
                artist_name: Some("Artist 2B".to_string()),
                extension: Some("old.ext".to_string()),
                ..Default::default()
            },
        ],
        genre: vec![],
    };

    sanitize_song(&mut song, "new.ext");

    assert_eq!(
        song.song.as_ref().unwrap().extension,
        Some("new.ext".to_string())
    );
    assert_eq!(
        song.song.as_ref().unwrap().id,
        Some("new.ext:song-2".to_string())
    );
    assert_eq!(
        song.album.as_ref().unwrap().extension,
        Some("new.ext".to_string())
    );
    assert_eq!(
        song.album.as_ref().unwrap().album_id,
        Some("new.ext:alb-2".to_string())
    );
    assert_len_eq_x!(&song.artists, 2);
    assert_eq!(song.artists[0].extension, Some("new.ext".to_string()));
    assert_eq!(
        song.artists[0].artist_id,
        Some("new.ext:art-2a".to_string())
    );
    assert_eq!(song.artists[1].extension, Some("new.ext".to_string()));
    assert_eq!(
        song.artists[1].artist_id,
        Some("new.ext:art-2b".to_string())
    );
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_sanitize_trait_search_result() {
    let resp = ExtensionCommandResponse {
        response: Some(extension_command_response::Response::RequestedSearchResult(
            RequestedSearchResultResponse {
                songs: vec![Song {
                    song: Some(InnerSong {
                        id: Some("s1".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
                playlists: vec![Playlist {
                    playlist_id: Some("p1".to_string()),
                    ..Default::default()
                }],
                artists: vec![Artist {
                    artist_id: Some("a1".to_string()),
                    ..Default::default()
                }],
                albums: vec![Album {
                    album_id: Some("al1".to_string()),
                    ..Default::default()
                }],
            },
        )),
    };

    let sanitized_resp = resp.sanitize("ext.pkg");
    let Some(extension_command_response::Response::RequestedSearchResult(search_res)) =
        sanitized_resp.response
    else {
        panic!("expected search result");
    };

    assert_eq!(
        search_res.songs[0].song.as_ref().unwrap().extension,
        Some("ext.pkg".to_string())
    );
    assert_eq!(
        search_res.songs[0].song.as_ref().unwrap().id,
        Some("ext.pkg:s1".to_string())
    );
    assert_eq!(
        search_res.playlists[0].extension,
        Some("ext.pkg".to_string())
    );
    assert_eq!(
        search_res.playlists[0].playlist_id,
        Some("ext.pkg:p1".to_string())
    );
    assert_eq!(search_res.artists[0].extension, Some("ext.pkg".to_string()));
    assert_eq!(
        search_res.artists[0].artist_id,
        Some("ext.pkg:a1".to_string())
    );
    assert_eq!(search_res.albums[0].extension, Some("ext.pkg".to_string()));
    assert_eq!(
        search_res.albums[0].album_id,
        Some("ext.pkg:al1".to_string())
    );
}
