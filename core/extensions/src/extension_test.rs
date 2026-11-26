use std::{
    fs,
    path::PathBuf,
    sync::{Arc, atomic::AtomicBool},
};

use assertables::{assert_is_empty, assert_len_eq_x};
use extensions_proto::moosync::types::{
    ExtensionCommandResponse, RequestedSearchResultResponse, extension_command_response,
};
use preferences_proto::moosync::types::PreferenceItem;
use rstest::{fixture, rstest};
use songs_proto::moosync::types::{
    Album, Artist, Genre, GetSongOptions, InnerSong, Playlist, Song,
};
use tempdir::TempDir;
use tracing_test::traced_test;

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
    fn get_albums(&self, _: &str, _: Album, _: bool) -> Result<Vec<Album>, ExtensionError> {
        Ok(vec![])
    }
    fn get_artists(&self, _: &str, _: Artist, _: bool) -> Result<Vec<Artist>, ExtensionError> {
        Ok(vec![])
    }
    fn get_genres(&self, _: &str, _: Genre, _: bool) -> Result<Vec<Genre>, ExtensionError> {
        Ok(vec![])
    }
    fn get_playlists(
        &self,
        _: &str,
        _: Playlist,
        _: bool,
    ) -> Result<Vec<Playlist>, ExtensionError> {
        Ok(vec![])
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
    fn add_playlist(&self, _: &str, _: Playlist) -> Result<String, ExtensionError> {
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
    fn set_account(
        &self,
        _: &str,
        _: extensions_proto::moosync::types::ExtensionAccountDetail,
    ) -> Result<bool, ExtensionError> {
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
    fn get_app_version(&self, _: &str) -> Result<String, ExtensionError> { Ok("1.0.0".to_string()) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_cache(&self, _: &str, _: &str) -> Result<Option<Vec<u8>>, ExtensionError> { Ok(None) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn set_cache(&self, _: &str, _: &str, _: Vec<u8>, _: u64) -> Result<bool, ExtensionError> {
        Ok(true)
    }
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
    let manifest = extensions_proto::moosync::types::ExtensionManifest {
        package_name: "unit.pkg".to_string(),
        display_name: "Unit Test Extension".to_string(),
        version: "1.0.0".to_string(),
        extension_entry: "main.wasm".to_string(),
        icon: "icon.png".to_string(),
        author: Some("Tester".to_string()),
        file_hashes: Default::default(),
        sha256: None,
        ..Default::default()
    };
    let manifest_path = temp_dir.path().join("manifest.pb");
    fs::write(&manifest_path, prost::Message::encode_to_vec(&manifest)).unwrap();
    let empty_wasm = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
    fs::write(temp_dir.path().join("main.wasm"), empty_wasm).unwrap();
    let lock_data = extensions_proto::moosync::types::ExtensionLockData {
        registry: "local".to_string(),
    };
    fs::write(
        temp_dir.path().join("extension.lock"),
        prost::Message::encode_to_vec(&lock_data),
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
fn test_extension_preferences(ext_context: TestExtContext) {
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
    assert_eq!(ext.get_lock_data().registry, "local");

    ext.register_ui_preferences(vec![PreferenceItem {
        id: "volume".to_string(),
        title: "Default Volume".to_string(),
        ..Default::default()
    }]);
    let details = ext.get_extension_detail();
    assert_len_eq_x!(&details.preferences, 1);
    assert_eq!(details.preferences[0].id, "volume");

    ext.unregister_ui_preferences(vec!["volume".to_string()]);
    let details_after = ext.get_extension_detail();
    assert_is_empty!(&details_after.preferences);

    let ordered_keys = ["zeta", "alpha", "mu", "beta", "omega"];
    let ordered_prefs: Vec<PreferenceItem> = ordered_keys
        .iter()
        .map(|k| PreferenceItem {
            id: (*k).to_string(),
            title: (*k).to_string(),
            ..Default::default()
        })
        .collect();
    ext.register_ui_preferences(ordered_prefs);
    ext.register_ui_preferences(vec![PreferenceItem {
        id: "alpha".to_string(),
        title: "Alpha Updated".to_string(),
        ..Default::default()
    }]);
    let ordered_details = ext.get_extension_detail();
    let actual_ids: Vec<&str> = ordered_details
        .preferences
        .iter()
        .map(|p| p.id.as_str())
        .collect();
    assert_eq!(actual_ids, ordered_keys);
    assert_eq!(ordered_details.preferences[1].title, "Alpha Updated");
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_extension_load_manifest_without_optional_fields() {
    let temp_dir = TempDir::new("moosync_ext_no_icon").expect("failed to create temp dir");
    let manifest = extensions_proto::moosync::types::ExtensionManifest {
        package_name: "sample.ext".to_string(),
        display_name: "Sample Extension".to_string(),
        version: "0.1.0".to_string(),
        extension_entry: "main.wasm".to_string(),
        file_hashes: Default::default(),
        sha256: None,
        ..Default::default()
    };
    let manifest_path = temp_dir.path().join("manifest.pb");
    fs::write(&manifest_path, prost::Message::encode_to_vec(&manifest)).unwrap();
    let empty_wasm = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
    fs::write(temp_dir.path().join("main.wasm"), empty_wasm).unwrap();
    let lock_data = extensions_proto::moosync::types::ExtensionLockData {
        registry: "dev".to_string(),
    };
    fs::write(
        temp_dir.path().join("extension.lock"),
        prost::Message::encode_to_vec(&lock_data),
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

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_extension_load_invalid_wasm_records_error() {
    let temp_dir = TempDir::new("moosync_ext_invalid_wasm").expect("failed to create temp dir");
    let manifest = extensions_proto::moosync::types::ExtensionManifest {
        package_name: "invalid.wasm.ext".to_string(),
        display_name: "Invalid WASM Extension".to_string(),
        version: "0.1.0".to_string(),
        extension_entry: "main.wasm".to_string(),
        file_hashes: Default::default(),
        sha256: None,
        ..Default::default()
    };
    let manifest_path = temp_dir.path().join("manifest.pb");
    fs::write(&manifest_path, prost::Message::encode_to_vec(&manifest)).unwrap();
    // Invalid wasm bytes
    fs::write(temp_dir.path().join("main.wasm"), b"not a wasm binary").unwrap();
    let cache_dir = temp_dir.path().join("cache");
    let reply = Arc::new(DummyReply);
    let has_started = Arc::new(AtomicBool::new(false));

    let ext = Extension::new(&manifest_path, reply, cache_dir, has_started);
    assert!(ext.is_ok());
    let ext = ext.unwrap();

    let detail = ext.get_extension_detail();
    assert_eq!(detail.package_name, "invalid.wasm.ext");
    assert!(!detail.has_started);
    assert!(detail.error.is_some());
    assert!(!detail.error.unwrap().is_empty());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_extension_group_with_order() {
    use futures::StreamExt;

    use crate::extension::ExtensionGroupExt;

    let temp_dir = TempDir::new("moosync_ext_group_order").unwrap();
    let manifest1 = extensions_proto::moosync::types::ExtensionManifest {
        package_name: "ext.one".to_string(),
        display_name: "Extension One".to_string(),
        version: "1.0.0".to_string(),
        extension_entry: "main.wasm".to_string(),
        file_hashes: Default::default(),
        sha256: None,
        scopes: vec![extensions_proto::moosync::types::ScopePermission {
            scope: extensions_proto::moosync::types::ExtensionProviderScope::Search as i32,
            justification: "Search".to_string(),
        }],
        ..Default::default()
    };
    let p1 = temp_dir.path().join("m1.pb");
    fs::write(&p1, prost::Message::encode_to_vec(&manifest1)).unwrap();
    fs::write(temp_dir.path().join("main.wasm"), b"wasm bytes").unwrap();

    let manifest2 = extensions_proto::moosync::types::ExtensionManifest {
        package_name: "ext.two".to_string(),
        display_name: "Extension Two".to_string(),
        version: "1.0.0".to_string(),
        extension_entry: "main.wasm".to_string(),
        file_hashes: Default::default(),
        sha256: None,
        scopes: vec![extensions_proto::moosync::types::ScopePermission {
            scope: extensions_proto::moosync::types::ExtensionProviderScope::Search as i32,
            justification: "Search".to_string(),
        }],
        ..Default::default()
    };
    let p2 = temp_dir.path().join("m2.pb");
    fs::write(&p2, prost::Message::encode_to_vec(&manifest2)).unwrap();

    let reply = Arc::new(DummyReply);
    let cache_dir = temp_dir.path().join("cache");
    let ext1 = Arc::new(
        Extension::new(
            &p1,
            reply.clone(),
            cache_dir.clone(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap(),
    );
    let ext2 =
        Arc::new(Extension::new(&p2, reply, cache_dir, Arc::new(AtomicBool::new(false))).unwrap());

    let extensions = [ext1, ext2];
    let mut stream = extensions.with(|ext| async move {
        if ext.manifest.package_name == "ext.one" {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            "ext.one"
        } else {
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            "ext.two"
        }
    });

    let first = stream.next().await.unwrap();
    let second = stream.next().await.unwrap();

    assert_eq!(first, "ext.two");
    assert_eq!(second, "ext.one");
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_extension_group_with_drop_cancels_pending() {
    use futures::StreamExt;

    use crate::extension::ExtensionGroupExt;

    let temp_dir = TempDir::new("moosync_ext_group_drop").unwrap();
    let manifest1 = extensions_proto::moosync::types::ExtensionManifest {
        package_name: "ext.fast".to_string(),
        display_name: "Fast Extension".to_string(),
        version: "1.0.0".to_string(),
        extension_entry: "main.wasm".to_string(),
        file_hashes: Default::default(),
        sha256: None,
        scopes: vec![extensions_proto::moosync::types::ScopePermission {
            scope: extensions_proto::moosync::types::ExtensionProviderScope::PlaybackDetails as i32,
            justification: "PlaybackDetails".to_string(),
        }],
        ..Default::default()
    };
    let p1 = temp_dir.path().join("fast.pb");
    fs::write(&p1, prost::Message::encode_to_vec(&manifest1)).unwrap();
    fs::write(temp_dir.path().join("main.wasm"), b"wasm bytes").unwrap();

    let manifest2 = extensions_proto::moosync::types::ExtensionManifest {
        package_name: "ext.slow".to_string(),
        display_name: "Slow Extension".to_string(),
        version: "1.0.0".to_string(),
        extension_entry: "main.wasm".to_string(),
        file_hashes: Default::default(),
        sha256: None,
        scopes: vec![extensions_proto::moosync::types::ScopePermission {
            scope: extensions_proto::moosync::types::ExtensionProviderScope::PlaybackDetails as i32,
            justification: "PlaybackDetails".to_string(),
        }],
        ..Default::default()
    };
    let p2 = temp_dir.path().join("slow.pb");
    fs::write(&p2, prost::Message::encode_to_vec(&manifest2)).unwrap();

    let reply = Arc::new(DummyReply);
    let cache_dir = temp_dir.path().join("cache");
    let ext1 = Arc::new(
        Extension::new(
            &p1,
            reply.clone(),
            cache_dir.clone(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap(),
    );
    let ext2 =
        Arc::new(Extension::new(&p2, reply, cache_dir, Arc::new(AtomicBool::new(false))).unwrap());

    struct TrackDrop(Arc<AtomicBool>);
    impl Drop for TrackDrop {
        #[tracing::instrument(level = "debug", skip_all)]
        fn drop(&mut self) { self.0.store(true, std::sync::atomic::Ordering::SeqCst); }
    }

    let slow_dropped = Arc::new(AtomicBool::new(false));

    let extensions = [ext1, ext2];
    {
        let slow_dropped_clone = slow_dropped.clone();
        let mut stream = extensions.with(move |ext| {
            let is_slow = ext.manifest.package_name == "ext.slow";
            let drop_tracker = if is_slow {
                Some(TrackDrop(slow_dropped_clone.clone()))
            } else {
                None
            };
            async move {
                let _drop = drop_tracker;
                if ext.manifest.package_name == "ext.fast" {
                    "fast_result"
                } else {
                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                    "slow_result"
                }
            }
        });

        let first = stream.next().await.unwrap();
        assert_eq!(first, "fast_result");
    }

    assert!(slow_dropped.load(std::sync::atomic::Ordering::SeqCst));
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_extension_provider_scopes_from_manifest() {
    let temp_dir = TempDir::new("moosync_ext_scopes").unwrap();
    let manifest = extensions_proto::moosync::types::ExtensionManifest {
        package_name: "scoped.ext".to_string(),
        display_name: "Scoped Extension".to_string(),
        version: "1.0.0".to_string(),
        extension_entry: "main.wasm".to_string(),
        file_hashes: Default::default(),
        sha256: None,
        scopes: vec![
            extensions_proto::moosync::types::ScopePermission {
                scope: extensions_proto::moosync::types::ExtensionProviderScope::Search as i32,
                justification: "Search".to_string(),
            },
            extensions_proto::moosync::types::ScopePermission {
                scope: extensions_proto::moosync::types::ExtensionProviderScope::Playlists as i32,
                justification: "Playlists".to_string(),
            },
        ],
        ..Default::default()
    };
    let manifest_path = temp_dir.path().join("manifest.pb");
    fs::write(&manifest_path, prost::Message::encode_to_vec(&manifest)).unwrap();
    fs::write(temp_dir.path().join("main.wasm"), b"wasm bytes").unwrap();

    let ext = Extension::new(
        &manifest_path,
        Arc::new(DummyReply),
        temp_dir.path().join("cache"),
        Arc::new(AtomicBool::new(false)),
    )
    .unwrap();

    let expected_scopes = vec![
        extensions_proto::moosync::types::ScopePermission {
            scope: extensions_proto::moosync::types::ExtensionProviderScope::Search as i32,
            justification: "Search".to_string(),
        },
        extensions_proto::moosync::types::ScopePermission {
            scope: extensions_proto::moosync::types::ExtensionProviderScope::Playlists as i32,
            justification: "Playlists".to_string(),
        },
    ];

    assert_eq!(ext.manifest.scopes, expected_scopes);
    let detail = ext.get_extension_detail();
    assert_eq!(detail.scopes, expected_scopes);
}

#[derive(Debug)]
struct MockExtCtx {
    should_fail: bool,
}

#[async_trait::async_trait]
impl crate::context::ExtensionContext for MockExtCtx {
    #[tracing::instrument(level = "debug", skip_all)]
    async fn execute_command(
        &self,
        command: extensions_proto::moosync::types::ExtensionCommand,
    ) -> Result<ExtensionCommandResponse, ExtensionError> {
        if self.should_fail {
            return Err(ExtensionError::InvalidResponse);
        }
        let resp = match command.event {
            Some(
                extensions_proto::moosync::types::extension_command::Event::RequestedPlaylists(_),
            ) => extension_command_response::Response::RequestedPlaylists(
                extensions_proto::moosync::types::RequestedPlaylistsResponse { playlists: vec![] },
            ),
            Some(
                extensions_proto::moosync::types::extension_command::Event::RequestedCoverArt(_),
            ) => extension_command_response::Response::RequestedCoverArt(
                extensions_proto::moosync::types::RequestedCoverArtResponse {
                    cover_urls: vec![
                        "https://example.com/cover_high.png".to_string(),
                        "https://example.com/cover_low.png".to_string(),
                    ],
                },
            ),
            Some(extensions_proto::moosync::types::extension_command::Event::RequestedLyrics(
                _,
            )) => extension_command_response::Response::RequestedLyrics(
                extensions_proto::moosync::types::RequestedLyricsResponse {
                    lyrics: Some(songs_proto::moosync::types::Lyrics {
                        is_synced: true,
                        lines: vec![songs_proto::moosync::types::LyricLine {
                            text: "Yesterday all my troubles seemed so far away".to_string(),
                            time_ms: 12500,
                        }],
                        ..Default::default()
                    }),
                },
            ),
            _ => extension_command_response::Response::RequestedPlaylists(
                extensions_proto::moosync::types::RequestedPlaylistsResponse { playlists: vec![] },
            ),
        };
        Ok(ExtensionCommandResponse {
            response: Some(resp),
        })
    }
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_async_context_lifecycle() {
    let context_immediate = crate::extension::AsyncContext::new();
    let mock = Arc::new(MockExtCtx { should_fail: false });
    context_immediate.set(Ok(mock.clone()));

    let wait_res = context_immediate.wait().await;
    assert!(wait_res.is_ok());

    let context_error = crate::extension::AsyncContext::new();
    context_error.set(Err("initialization failed".to_string()));
    let err_res = context_error.wait().await;
    assert!(err_res.is_err());

    let context_delayed = Arc::new(crate::extension::AsyncContext::new());
    let context_clone = context_delayed.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        context_clone.set(Ok(mock));
    });

    let async_res = context_delayed.wait().await;
    assert!(async_res.is_ok());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_extension_accounts_and_delegated_commands() {
    let temp_dir = TempDir::new("moosync_ext_delegation").unwrap();
    let manifest = extensions_proto::moosync::types::ExtensionManifest {
        package_name: "delegated.ext".to_string(),
        display_name: "Delegated Extension".to_string(),
        version: "1.0.0".to_string(),
        extension_entry: "main.wasm".to_string(),
        file_hashes: Default::default(),
        sha256: None,
        ..Default::default()
    };
    let manifest_path = temp_dir.path().join("manifest.pb");
    fs::write(&manifest_path, prost::Message::encode_to_vec(&manifest)).unwrap();
    fs::write(temp_dir.path().join("main.wasm"), b"wasm bytes").unwrap();

    let ext = Extension::new(
        &manifest_path,
        Arc::new(DummyReply),
        temp_dir.path().join("cache"),
        Arc::new(AtomicBool::new(false)),
    )
    .unwrap();

    let account = extensions_proto::moosync::types::ExtensionAccountDetail {
        id: "acc_99".to_string(),
        name: "My Account".to_string(),
        ..Default::default()
    };
    ext.set_account(account);
    let accounts = ext.get_accounts();
    assert_len_eq_x!(&accounts, 1);
    assert_eq!(accounts[0].package_name, "delegated.ext");

    ext.context
        .set(Ok(Arc::new(MockExtCtx { should_fail: false })));

    let playlists_res = ext
        .get_playlists(extensions_proto::moosync::types::RequestedPlaylistsRequest::default())
        .await;
    assert!(playlists_res.is_ok());

    let invalid_res = ext
        .get_artist_songs(extensions_proto::moosync::types::RequestedArtistSongsRequest::default())
        .await;
    assert!(invalid_res.is_err());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_extension_get_cover_art_api() {
    let temp_dir = TempDir::new("moosync_ext_cover_art_test").unwrap();
    let manifest = extensions_proto::moosync::types::ExtensionManifest {
        package_name: "cover_art.test".to_string(),
        display_name: "Cover Art Test Extension".to_string(),
        version: "1.0.0".to_string(),
        extension_entry: "main.wasm".to_string(),
        file_hashes: Default::default(),
        sha256: None,
        scopes: vec![extensions_proto::moosync::types::ScopePermission {
            scope: extensions_proto::moosync::types::ExtensionProviderScope::CoverArt as i32,
            justification: "CoverArt".to_string(),
        }],
        ..Default::default()
    };
    let manifest_path = temp_dir.path().join("manifest.pb");
    fs::write(&manifest_path, prost::Message::encode_to_vec(&manifest)).unwrap();
    fs::write(temp_dir.path().join("main.wasm"), b"wasm bytes").unwrap();

    let ext = Extension::new(
        &manifest_path,
        Arc::new(DummyReply),
        temp_dir.path().join("cache"),
        Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    ext.context
        .set(Ok(Arc::new(MockExtCtx { should_fail: false })));

    let test_song = Song {
        song: Some(InnerSong {
            id: Some("song_cover_123".to_string()),
            title: Some("Yesterday".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let req = extensions_proto::moosync::types::RequestedCoverArtRequest {
        item: Some(
            extensions_proto::moosync::types::requested_cover_art_request::Item::Song(test_song),
        ),
    };

    let result = ext.get_cover_art(req).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.cover_urls.len(), 2);
    assert_eq!(response.cover_urls[0], "https://example.com/cover_high.png");
    assert_eq!(response.cover_urls[1], "https://example.com/cover_low.png");

    let failing_ext = Extension::new(
        &manifest_path,
        Arc::new(DummyReply),
        temp_dir.path().join("cache_failing"),
        Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    failing_ext
        .context
        .set(Ok(Arc::new(MockExtCtx { should_fail: true })));

    let fail_result = failing_ext
        .get_cover_art(extensions_proto::moosync::types::RequestedCoverArtRequest::default())
        .await;

    assert!(fail_result.is_err());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_extension_get_lyrics_api() {
    let temp_dir = TempDir::new("moosync_ext_lyrics_test").unwrap();
    let manifest = extensions_proto::moosync::types::ExtensionManifest {
        package_name: "lyrics.test".to_string(),
        display_name: "Lyrics Test Extension".to_string(),
        version: "1.0.0".to_string(),
        extension_entry: "main.wasm".to_string(),
        file_hashes: Default::default(),
        sha256: None,
        scopes: vec![extensions_proto::moosync::types::ScopePermission {
            scope: extensions_proto::moosync::types::ExtensionProviderScope::Lyrics as i32,
            justification: "Lyrics".to_string(),
        }],
        ..Default::default()
    };
    let manifest_path = temp_dir.path().join("manifest.pb");
    fs::write(&manifest_path, prost::Message::encode_to_vec(&manifest)).unwrap();
    fs::write(temp_dir.path().join("main.wasm"), b"wasm bytes").unwrap();

    let ext = Extension::new(
        &manifest_path,
        Arc::new(DummyReply),
        temp_dir.path().join("cache"),
        Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    ext.context
        .set(Ok(Arc::new(MockExtCtx { should_fail: false })));

    let test_song = Song {
        song: Some(InnerSong {
            id: Some("song_lyrics_123".to_string()),
            title: Some("Yesterday".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let req = extensions_proto::moosync::types::RequestedLyricsRequest {
        song: Some(test_song),
    };

    let result = ext.get_lyrics(req).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.lyrics.is_some());
    let lyrics = response.lyrics.unwrap();
    assert!(lyrics.is_synced);
    assert_eq!(lyrics.lines.len(), 1);
    assert_eq!(
        lyrics.lines[0].text,
        "Yesterday all my troubles seemed so far away"
    );
    assert_eq!(lyrics.lines[0].time_ms, 12500);

    let failing_ext = Extension::new(
        &manifest_path,
        Arc::new(DummyReply),
        temp_dir.path().join("cache_failing"),
        Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    failing_ext
        .context
        .set(Ok(Arc::new(MockExtCtx { should_fail: true })));

    let fail_result = failing_ext
        .get_lyrics(extensions_proto::moosync::types::RequestedLyricsRequest::default())
        .await;

    assert!(fail_result.is_err());
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_extension_player_data_timer_advances_and_pauses() {
    let data = crate::extension::ExtensionPlayerData::new("test.player");

    data.set_playing();
    std::thread::sleep(std::time::Duration::from_millis(30));
    let pos_while_playing = data.current_position();
    data.set_paused(extensions_proto::moosync::types::PlayerState::Paused);
    let pos_on_pause = data.current_position();
    std::thread::sleep(std::time::Duration::from_millis(30));
    let pos_after_pause = data.current_position();

    assert!(pos_while_playing >= std::time::Duration::from_millis(20));
    assert!(pos_on_pause >= pos_while_playing);
    assert_eq!(pos_on_pause, pos_after_pause);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_extension_player_data_set_position_updates_base() {
    let data = crate::extension::ExtensionPlayerData::new("test.player");
    data.set_playing();

    data.set_position(std::time::Duration::from_secs(42));
    let pos = data.current_position();

    assert!(pos >= std::time::Duration::from_secs(42));
    assert!(pos < std::time::Duration::from_secs(43));
}
