use std::{fs, sync::Arc};

use assertables::{assert_is_empty, assert_len_eq_x};
use rstest::{fixture, rstest};
use songs_proto::moosync::types::{Album, Artist, Genre, GetSongOptions, Playlist, Song};
use tempdir::TempDir;
use tracing_test::traced_test;

use crate::{ReplyHandler, errors::ExtensionError, ext_runner::ExtensionHandlerInner};

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
        _: Vec<preferences_proto::moosync::types::PreferenceItem>,
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

struct TestRunnerContext {
    pub _temp_dir: TempDir,
    pub runner: ExtensionHandlerInner,
}

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn runner_context() -> TestRunnerContext {
    let temp_dir = TempDir::new("moosync_ext_runner").expect("failed to create temp dir");
    let ext_dir = temp_dir.path().join("exts");
    let cache_dir = temp_dir.path().join("cache");
    fs::create_dir_all(&ext_dir).unwrap();
    fs::create_dir_all(&cache_dir).unwrap();

    let runner = ExtensionHandlerInner::new(ext_dir, cache_dir);
    TestRunnerContext {
        _temp_dir: temp_dir,
        runner,
    }
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_ext_runner_get_installed_extensions_empty(runner_context: TestRunnerContext) {
    let TestRunnerContext { runner, .. } = runner_context;

    let list = runner.get_installed_extensions();

    assert_is_empty!(&list);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_ext_runner_dynamic_extension_installation(runner_context: TestRunnerContext) {
    let TestRunnerContext { runner, _temp_dir } = runner_context;
    let ext_dir = _temp_dir.path().join("exts");
    let ext1_path = ext_dir.join("ext_one");
    fs::create_dir_all(&ext1_path).unwrap();

    let manifest1 = extensions_proto::moosync::types::ExtensionManifest {
        name: "ext.one".to_string(),
        display_name: "Extension One".to_string(),
        version: "1.0.0".to_string(),
        extension_entry: "main.wasm".to_string(),
        scopes: vec![extensions_proto::moosync::types::ScopePermission {
            scope: extensions_proto::moosync::types::ExtensionProviderScope::Search as i32,
            justification: "Search".to_string(),
        }],
        ..Default::default()
    };
    fs::write(
        ext1_path.join("manifest.pb"),
        prost::Message::encode_to_vec(&manifest1),
    )
    .unwrap();
    fs::write(ext1_path.join("main.wasm"), b"wasm bytes").unwrap();

    let reply_handler = Arc::new(DummyReply);
    runner.spawn_extensions(reply_handler.clone());

    let installed_first = runner.get_installed_extensions();
    assert_len_eq_x!(&installed_first, 1);
    assert_eq!(installed_first[0].package_name, "ext.one");

    let ext1_arc_first = runner.get_extension("ext.one").unwrap();

    let ext2_path = ext_dir.join("ext_two");
    fs::create_dir_all(&ext2_path).unwrap();

    let manifest2 = extensions_proto::moosync::types::ExtensionManifest {
        name: "ext.two".to_string(),
        display_name: "Extension Two".to_string(),
        version: "1.0.0".to_string(),
        extension_entry: "main.wasm".to_string(),
        scopes: vec![extensions_proto::moosync::types::ScopePermission {
            scope: extensions_proto::moosync::types::ExtensionProviderScope::Playlists as i32,
            justification: "Playlists".to_string(),
        }],
        ..Default::default()
    };
    fs::write(
        ext2_path.join("manifest.pb"),
        prost::Message::encode_to_vec(&manifest2),
    )
    .unwrap();
    fs::write(ext2_path.join("main.wasm"), b"wasm bytes").unwrap();

    runner.spawn_extensions(reply_handler);

    let installed_second = runner.get_installed_extensions();
    assert_len_eq_x!(&installed_second, 2);

    let ext1_arc_second = runner.get_extension("ext.one").unwrap();
    let ext2_arc = runner.get_extension("ext.two").unwrap();

    assert!(Arc::ptr_eq(&ext1_arc_first, &ext1_arc_second));
    assert_eq!(ext2_arc.manifest.name, "ext.two");

    runner.remove_extension("ext.one");
    assert_len_eq_x!(&runner.get_installed_extensions(), 1);
    assert!(runner.get_extension("ext.one").is_err());
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_ext_runner_missing_and_direct_manifests(runner_context: TestRunnerContext) {
    let TestRunnerContext { runner, _temp_dir } = runner_context;
    let ext_dir = _temp_dir.path().join("exts");

    let manifest = extensions_proto::moosync::types::ExtensionManifest {
        name: "direct.ext".to_string(),
        display_name: "Direct Ext".to_string(),
        version: "1.0.0".to_string(),
        ..Default::default()
    };
    fs::write(
        ext_dir.join("manifest.pb"),
        prost::Message::encode_to_vec(&manifest),
    )
    .unwrap();

    let invalid_dir = ext_dir.join("corrupt_sub");
    fs::create_dir_all(&invalid_dir).unwrap();
    fs::write(invalid_dir.join("manifest.pb"), b"corrupt bytes").unwrap();

    let reply_handler = Arc::new(DummyReply);
    runner.spawn_extensions(reply_handler);

    let non_existent_runner = ExtensionHandlerInner::new(
        _temp_dir.path().join("does_not_exist"),
        _temp_dir.path().join("cache"),
    );
    let missing_installed = non_existent_runner.get_installed_extensions();
    assert_is_empty!(&missing_installed);
}
