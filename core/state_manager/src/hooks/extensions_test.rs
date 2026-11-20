use assertables::assert_ok;
use rstest::{fixture, rstest};
use tempdir::TempDir;
use tracing_test::traced_test;
use types::plugin::PluginContext;

use crate::{
    StateManager,
    hooks::{Hook, extensions::ExtensionsHook},
};

struct TestSmContext {
    pub _temp_dir: TempDir,
    pub sm: StateManager,
}

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn sm_context() -> TestSmContext {
    let temp_dir = TempDir::new("moosync_sm_ext_test").expect("failed to create temp dir");
    let test_dir = temp_dir.path().to_path_buf();
    let context = PluginContext {
        data_dir: test_dir.clone(),
        cache_dir: test_dir.clone(),
        tmp_dir: test_dir.clone(),
        #[cfg(target_os = "android")]
        android_context: types::android::AndroidJNIContext::default(),
    };
    let sm = StateManager::new_with_context(context).expect("failed to create state manager");
    TestSmContext {
        _temp_dir: temp_dir,
        sm,
    }
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_extensions_hook_on_startup(sm_context: TestSmContext) {
    let TestSmContext { sm, .. } = sm_context;
    let hook = ExtensionsHook::new();
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    let _handle = sm
        .get_extension_handler()
        .on_extensions_updated
        .listen(move |_| {
            let _ = tx.try_send(());
        });

    assert_ok!(hook.on_startup(&sm).await);

    let received = tokio::time::timeout(std::time::Duration::from_millis(500), rx.recv()).await;
    assert!(received.is_ok());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_extensions_hook_on_songs_added_listener(sm_context: TestSmContext) {
    let TestSmContext { sm, .. } = sm_context;
    let hook = ExtensionsHook::new();
    assert_ok!(hook.on_startup(&sm).await);

    let song = songs_proto::moosync::types::Song {
        song: Some(songs_proto::moosync::types::InnerSong {
            id: Some("hook_song_1".into()),
            title: Some("Hook Song".into()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let inserted = sm.get_database().insert_songs(vec![song]).await;
    assert_ok!(inserted);
}
