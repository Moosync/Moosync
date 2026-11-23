use assertables::assert_ok;
use rstest::rstest;
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use songs_proto::moosync::types::Playlist;
use tracing_test::traced_test;

use crate::{
    ContextMenuCallbacks, MainWindow, PlaylistModel, PlaylistsPageProps,
    main_content::playlists::{PlaylistListProvider, PlaylistsPageHandler},
    pages::PageHandler,
    test_utils::{TestSlintSmContext, main_window, state_manager_fixture},
    utils::EntityListProvider,
};

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_playlist_list_provider_name() {
    let name = PlaylistListProvider::name();

    assert_eq!(name, "Playlists");
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_playlist_list_provider_to_model() {
    let playlist = Playlist {
        playlist_name: "My Test Playlist".into(),
        ..Default::default()
    };

    let model = PlaylistListProvider::to_model(playlist);

    assert_eq!(model.title.as_str(), "My Test Playlist");
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_playlist_list_provider_set_models(main_window: MainWindow) {
    let dummy_playlists = vec![PlaylistModel::default(), PlaylistModel::default()];
    let model = ModelRc::new(VecModel::from(dummy_playlists));

    PlaylistListProvider::set_models(&main_window, model);

    assert_eq!(
        main_window
            .global::<PlaylistsPageProps>()
            .get_playlists()
            .row_count(),
        2
    );
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_playlist_list_provider_clear_models(main_window: MainWindow) {
    let dummy_playlists = vec![PlaylistModel::default(), PlaylistModel::default()];
    let model = ModelRc::new(VecModel::from(dummy_playlists));
    PlaylistListProvider::set_models(&main_window, model);

    PlaylistListProvider::clear_models(&main_window);

    assert_eq!(
        main_window
            .global::<PlaylistsPageProps>()
            .get_playlists()
            .row_count(),
        0
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_playlist_list_provider_fetch_entities_success(
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let playlist = Playlist {
        playlist_name: "My Test Playlist".into(),
        ..Default::default()
    };
    let db = sm.get_database();
    assert_ok!(db.create_playlist_with_songs(playlist, &[]).await);

    let entities_res = PlaylistListProvider::fetch_entities(&sm).await;

    assert_ok!(&entities_res);
    let entities = entities_res.unwrap();
    assert_eq!(entities.len(), 1);
    assert_eq!(entities[0].playlist_name.as_str(), "My Test Playlist");
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_playlists_page_handler_on_hide(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window = main_window;
    let state_manager = sm;
    let dummy_playlists = vec![PlaylistModel::default(), PlaylistModel::default()];
    main_window
        .global::<PlaylistsPageProps>()
        .set_playlists(ModelRc::new(VecModel::from(dummy_playlists)));
    let handler = PlaylistsPageHandler::new(&main_window, &state_manager);

    handler.on_hide();

    assert_eq!(
        main_window
            .global::<PlaylistsPageProps>()
            .get_playlists()
            .row_count(),
        0
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_playlists_page_handler_initialize_and_context_menu(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window = main_window;
    let state_manager = sm;
    let handler = PlaylistsPageHandler::new(&main_window, &state_manager);
    handler.initialize();

    let playlist_model = PlaylistModel {
        id: "p1".into(),
        title: "Playlist 1".into(),
        ..Default::default()
    };
    let models = ModelRc::new(VecModel::from(vec![playlist_model]));

    let callbacks = main_window.global::<ContextMenuCallbacks>();
    let items = callbacks.invoke_get_playlist_menu_items(models.clone());
    assert!(items.row_count() > 0);

    callbacks.invoke_playlist_action(models.clone(), "other_action".into());
    callbacks.invoke_playlist_action(models, "extension:sample_pkg:test_action".into());
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_playlists_page_handler_delete_action(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window = main_window;
    let state_manager = sm;
    let playlist = Playlist {
        playlist_name: "To Delete".into(),
        ..Default::default()
    };
    let db = state_manager.get_database();
    let id = db.create_playlist(playlist).await.unwrap();

    PlaylistsPageHandler::handle_playlist_action(
        main_window.as_weak(),
        state_manager.clone(),
        vec![id.clone()],
        "delete_playlist".into(),
    )
    .await;

    let local = PlaylistListProvider::get_local_playlists(&state_manager)
        .await
        .unwrap();
    assert!(local.iter().all(|p| p.playlist_id.as_deref() != Some(&id)));
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_playlist_list_provider_set_loading(main_window: MainWindow) {
    PlaylistListProvider::set_loading(&main_window, true);
    assert!(main_window.global::<PlaylistsPageProps>().get_is_loading());

    PlaylistListProvider::clear_models(&main_window);
    assert!(!main_window.global::<PlaylistsPageProps>().get_is_loading());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_playlists_page_handler_on_show_and_hide_loading(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let handler = PlaylistsPageHandler::new(&main_window, &sm);

    handler.on_show();
    assert!(main_window.global::<PlaylistsPageProps>().get_is_loading());

    handler.on_hide();
    assert!(!main_window.global::<PlaylistsPageProps>().get_is_loading());
}
