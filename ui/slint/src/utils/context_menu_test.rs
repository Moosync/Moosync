use slint::{Model, ModelRc, VecModel};
use state_manager::StateManager;
use tempdir::TempDir;
use types::plugin::PluginContext;

use super::{
    build_queue_context_menu_items, build_song_context_menu_items, default_empty_icon,
    make_context_menu_item, models::IntoVec,
};
use crate::{ContextMenuItem, MainWindow, test_utils::run_async_test};

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_context_menu_item_vec_from_model_rc() {
    let items = vec![
        make_context_menu_item("play_now", "Play Now", default_empty_icon()),
        make_context_menu_item("add_to_queue", "Add to Queue", default_empty_icon()),
    ];

    let model_rc = ModelRc::new(VecModel::from(items));
    let converted: Vec<ContextMenuItem> = model_rc.into_vec();

    assert_eq!(converted.len(), 2);
    assert_eq!(converted[0].action_id, "play_now");
    assert_eq!(converted[0].title, "Play Now");
    assert_eq!(converted[1].action_id, "add_to_queue");
    assert_eq!(converted[1].title, "Add to Queue");
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_build_song_context_menu_items() {
    run_async_test(|| async move {
        let main_window = MainWindow::new().unwrap();
        let tmp = TempDir::new("ctx_song_test").unwrap();
        let test_dir = tmp.path().to_path_buf();
        let context = PluginContext {
            data_dir: test_dir.clone(),
            cache_dir: test_dir.clone(),
            tmp_dir: test_dir.clone(),
            #[cfg(target_os = "android")]
            android_context: types::android::AndroidJNIContext::default(),
        };
        let state_manager = StateManager::new_with_context(context).unwrap();
        let song_models = ModelRc::default();

        let items = build_song_context_menu_items(&main_window, &state_manager, &song_models);

        assert!(items.row_count() > 0);
    });
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_build_queue_context_menu_items() {
    run_async_test(|| async move {
        let main_window = MainWindow::new().unwrap();
        let tmp = TempDir::new("ctx_queue_test").unwrap();
        let test_dir = tmp.path().to_path_buf();
        let context = PluginContext {
            data_dir: test_dir.clone(),
            cache_dir: test_dir.clone(),
            tmp_dir: test_dir.clone(),
            #[cfg(target_os = "android")]
            android_context: types::android::AndroidJNIContext::default(),
        };
        let state_manager = StateManager::new_with_context(context).unwrap();
        let song_models = ModelRc::default();

        let items = build_queue_context_menu_items(&main_window, &state_manager, &song_models, 0);

        assert!(items.row_count() > 0);
    });
}
