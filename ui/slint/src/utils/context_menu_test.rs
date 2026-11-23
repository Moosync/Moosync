use rstest::rstest;
use slint::{ComponentHandle, Model, ModelRc};
use tracing_test::traced_test;

use super::{
    ACTION_ADD_TO_PLAYLIST_NEW, ACTION_ADD_TO_QUEUE, ACTION_CLEAR_QUEUE_AND_PLAY, ACTION_PLAY_NEXT,
    ACTION_PLAY_NOW, ACTION_REMOVE_FROM_PLAYLIST, PREFIX_ADD_TO_PLAYLIST, PREFIX_EXTENSION,
    PREFIX_GOTO_ALBUM, PREFIX_GOTO_ARTIST, build_album_context_menu_items,
    build_artist_context_menu_items, build_genre_context_menu_items,
    build_playlist_context_menu_items, build_queue_context_menu_items,
    build_song_context_menu_items, dispatch_album_context_action, dispatch_artist_context_action,
    dispatch_genre_context_action, handle_extension_context_action,
};
use crate::{
    AlbumModel, ArtistModel, GenreModel, MainWindow, PlaylistModel,
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
    let state_manager = sm;
    let song_models = ModelRc::default();

    let items = build_song_context_menu_items(&main_window, state_manager.clone(), &song_models);

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
    let state_manager = sm;
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
    let state_manager = sm;

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

    let items = build_song_context_menu_items(&main_window, state_manager.clone(), &song_models);

    assert!(items.row_count() >= 5);
    for i in 0..items.row_count() {
        let item = items.row_data(i).unwrap();
        assert!(item.icon.size().width > 0);
        for j in 0..item.sub_items.row_count() {
            let sub = item.sub_items.row_data(j).unwrap();
            assert!(sub.icon.size().width > 0);
        }
    }
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
    let state_manager = sm;

    let song = crate::SongModel {
        id: "song_1".into(),
        title: "Song 1".into(),
        ..Default::default()
    };
    let song_models = ModelRc::new(slint::VecModel::from(vec![song]));
    let weak = main_window.as_weak();

    super::dispatch_song_context_action(&weak, &state_manager, &song_models, ACTION_PLAY_NOW);
    super::dispatch_song_context_action(&weak, &state_manager, &song_models, ACTION_PLAY_NEXT);
    super::dispatch_song_context_action(
        &weak,
        &state_manager,
        &song_models,
        ACTION_CLEAR_QUEUE_AND_PLAY,
    );
    super::dispatch_song_context_action(&weak, &state_manager, &song_models, ACTION_ADD_TO_QUEUE);

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
    let state_manager = sm;

    let song = crate::SongModel {
        id: "song_1".into(),
        title: "Song 1".into(),
        ..Default::default()
    };
    let song_models = ModelRc::new(slint::VecModel::from(vec![song]));
    let weak = main_window.as_weak();

    super::dispatch_song_context_action(
        &weak,
        &state_manager,
        &song_models,
        &format!("{PREFIX_ADD_TO_PLAYLIST}p_test"),
    );
    super::dispatch_song_context_action(
        &weak,
        &state_manager,
        &song_models,
        ACTION_REMOVE_FROM_PLAYLIST,
    );
    super::dispatch_song_context_action(
        &weak,
        &state_manager,
        &song_models,
        &format!("{PREFIX_GOTO_ALBUM}alb_test"),
    );
    super::dispatch_song_context_action(
        &weak,
        &state_manager,
        &song_models,
        &format!("{PREFIX_GOTO_ARTIST}art_test"),
    );

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_build_playlist_context_menu_items(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager = sm;
    let playlist = PlaylistModel {
        id: "pl_1".into(),
        title: "Favorites".into(),
        ..Default::default()
    };
    let playlist_models = ModelRc::new(slint::VecModel::from(vec![playlist]));

    let items = build_playlist_context_menu_items(&main_window, state_manager, &playlist_models);

    assert_eq!(items.row_count(), 3);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_handle_extension_context_action_prefixes(
    _main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager = sm;

    let handled_native = handle_extension_context_action(&state_manager, ACTION_PLAY_NOW).await;
    assert!(!handled_native);

    let handled_bad_prefix =
        handle_extension_context_action(&state_manager, &format!("{PREFIX_EXTENSION}no_colon"))
            .await;
    assert!(!handled_bad_prefix);

    let handled_ext = handle_extension_context_action(
        &state_manager,
        &format!("{PREFIX_EXTENSION}unknown_pkg:my_action"),
    )
    .await;
    assert!(handled_ext);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_dispatch_song_context_action_new_playlist(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager = sm;

    let song = crate::SongModel {
        id: "song_new_pl".into(),
        title: "Song New PL".into(),
        ..Default::default()
    };
    let song_models = ModelRc::new(slint::VecModel::from(vec![song]));
    let weak = main_window.as_weak();

    super::dispatch_song_context_action(
        &weak,
        &state_manager,
        &song_models,
        ACTION_ADD_TO_PLAYLIST_NEW,
    );

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_build_entity_context_menu_items(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager = sm;

    let album_models = ModelRc::new(slint::VecModel::from(vec![AlbumModel::default()]));
    let artist_models = ModelRc::new(slint::VecModel::from(vec![ArtistModel::default()]));
    let genre_models = ModelRc::new(slint::VecModel::from(vec![GenreModel::default()]));

    let album_items = build_album_context_menu_items(&main_window, &state_manager, &album_models);
    let artist_items =
        build_artist_context_menu_items(&main_window, &state_manager, &artist_models);
    let genre_items = build_genre_context_menu_items(&main_window, &state_manager, &genre_models);

    assert_eq!(album_items.row_count(), 4);
    assert_eq!(artist_items.row_count(), 4);
    assert_eq!(genre_items.row_count(), 4);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_dispatch_entity_context_actions(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager = sm;
    let weak = main_window.as_weak();

    let album_models = ModelRc::new(slint::VecModel::from(vec![AlbumModel::default()]));
    let artist_models = ModelRc::new(slint::VecModel::from(vec![ArtistModel::default()]));
    let genre_models = ModelRc::new(slint::VecModel::from(vec![GenreModel::default()]));

    dispatch_album_context_action(
        &weak,
        &state_manager,
        &album_models,
        ACTION_ADD_TO_PLAYLIST_NEW,
    );
    dispatch_artist_context_action(&weak, &state_manager, &artist_models, ACTION_PLAY_NOW);
    dispatch_genre_context_action(&weak, &state_manager, &genre_models, ACTION_ADD_TO_QUEUE);

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}
