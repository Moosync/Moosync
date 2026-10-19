use rstest::rstest;
use slint::{ComponentHandle, Model, ModelRc};
use tracing_test::traced_test;

use super::{build_queue_context_menu_items, build_song_context_menu_items};
use crate::{
    MainWindow,
    test_utils::{TestSlintSmContext, main_window, state_manager_fixture},
};

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_build_song_context_menu_items(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager = Box::leak(Box::new(sm));
    let song_models = ModelRc::default();

    let items = build_song_context_menu_items(&main_window, state_manager, &song_models);

    assert_eq!(items.row_count(), 4);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_build_queue_context_menu_items(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager = Box::leak(Box::new(sm));
    let song_models = ModelRc::default();

    let items = build_queue_context_menu_items(&main_window, state_manager, &song_models, 0);

    assert_eq!(items.row_count(), 2);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_build_song_context_menu_items_with_album_and_artists(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager = Box::leak(Box::new(sm));

    let artist = crate::ArtistModel {
        id: "artist_1".into(),
        title: "Great Artist".into(),
        ..Default::default()
    };
    let song = crate::SongModel {
        id: "song_1".into(),
        title: "Song 1".into(),
        album_name: "Great Album".into(),
        artists: ModelRc::new(slint::VecModel::from(vec![artist])),
        ..Default::default()
    };
    let song_models = ModelRc::new(slint::VecModel::from(vec![song]));

    let items = build_song_context_menu_items(&main_window, state_manager, &song_models);

    assert!(items.row_count() >= 5);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_dispatch_song_context_action_playback(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager = Box::leak(Box::new(sm));

    let song = crate::SongModel {
        id: "song_1".into(),
        title: "Song 1".into(),
        ..Default::default()
    };
    let song_models = ModelRc::new(slint::VecModel::from(vec![song]));
    let weak = main_window.as_weak();

    super::dispatch_song_context_action(&weak, state_manager, &song_models, "play_now");
    super::dispatch_song_context_action(&weak, state_manager, &song_models, "play_next");
    super::dispatch_song_context_action(&weak, state_manager, &song_models, "clear_queue_and_play");
    super::dispatch_song_context_action(&weak, state_manager, &song_models, "add_to_queue");

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let queue = state_manager.get_player_handler().get_queue();
    assert!(!queue.is_empty());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_dispatch_song_context_action_playlist_and_nav(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager = Box::leak(Box::new(sm));

    let song = crate::SongModel {
        id: "song_1".into(),
        title: "Song 1".into(),
        ..Default::default()
    };
    let song_models = ModelRc::new(slint::VecModel::from(vec![song]));
    let weak = main_window.as_weak();

    super::dispatch_song_context_action(
        &weak,
        state_manager,
        &song_models,
        "add_to_playlist:p_test",
    );
    super::dispatch_song_context_action(&weak, state_manager, &song_models, "remove_from_playlist");
    super::dispatch_song_context_action(&weak, state_manager, &song_models, "goto_album:alb_test");
    super::dispatch_song_context_action(&weak, state_manager, &song_models, "goto_artist:art_test");

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}
