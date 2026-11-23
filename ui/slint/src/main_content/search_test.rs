use extensions_proto::moosync::types::{ExtensionDetail, ExtensionProviderScope, ScopePermission};
use rstest::rstest;
use slint::{ComponentHandle, Model, ModelRc};
use songs_proto::moosync::types::{
    InnerSong, SearchResult as ProtoSearchResult, Song as ProtoSong,
};
use tracing_test::traced_test;
use types::prelude::ScopePermissionExt;

use crate::{
    AppCallbacks, MainWindow, SearchPageProps,
    main_content::search::SearchPageHandler,
    pages::PageHandler,
    test_utils::{TestSlintSmContext, main_window, state_manager_fixture},
};

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_search_page_handler_search_whitespace_clears_results(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window = main_window;
    let state_manager = sm;
    let handler = SearchPageHandler::new(&main_window, &state_manager);
    handler.initialize();

    main_window
        .global::<AppCallbacks>()
        .invoke_search_term_changed("   ".into());

    let results = main_window
        .global::<SearchPageProps>()
        .get_provider_results();
    assert_eq!(results.row_count(), 1);
    assert_eq!(results.row_data(0).unwrap().songs.row_count(), 0);
    assert!(!results.row_data(0).unwrap().loading);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_search_page_handler_search_empty_clears_results(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window = main_window;
    let state_manager = sm;
    let handler = SearchPageHandler::new(&main_window, &state_manager);
    handler.initialize();

    main_window
        .global::<AppCallbacks>()
        .invoke_search_term_changed("".into());

    let results = main_window
        .global::<SearchPageProps>()
        .get_provider_results();
    assert_eq!(results.row_count(), 1);
    assert_eq!(results.row_data(0).unwrap().songs.row_count(), 0);
    assert!(!results.row_data(0).unwrap().loading);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_search_page_handler_append_search_result(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window = main_window;
    let state_manager = sm;

    let res = ProtoSearchResult {
        songs: vec![ProtoSong {
            song: Some(InnerSong {
                id: Some("search_song_1".to_string()),
                title: Some("Search Song".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        }],
        ..Default::default()
    };

    main_window
        .global::<SearchPageProps>()
        .set_provider_results(ModelRc::default());

    let cache_dir = state_manager.get_cache_dir();
    SearchPageHandler::update_provider_result(&main_window, &cache_dir, res, None, false, false);

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    assert_eq!(
        main_window
            .global::<SearchPageProps>()
            .get_provider_results()
            .row_count(),
        1
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_search_page_handler_perform_search_and_accounts(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window = main_window;
    let state_manager = sm;
    let handler = SearchPageHandler::new(&main_window, &state_manager);
    handler.initialize();

    main_window
        .global::<SearchPageProps>()
        .set_search_query("test query".into());

    main_window
        .global::<AppCallbacks>()
        .invoke_search_term_changed("rock".into());

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    state_manager
        .get_extension_handler()
        .on_accounts_updated
        .emit(None);

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_search_page_handler_initial_providers(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window = main_window;
    let state_manager = sm;

    let handler = SearchPageHandler::new(&main_window, &state_manager);
    handler.initialize();

    let initial_providers = main_window
        .global::<SearchPageProps>()
        .get_provider_results();
    assert_eq!(initial_providers.row_count(), 1);
    assert_eq!(initial_providers.row_data(0).unwrap().extension, "");
    assert!(!initial_providers.row_data(0).unwrap().loading);
    assert_eq!(initial_providers.row_data(0).unwrap().songs.row_count(), 0);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_search_page_handler_update_provider_result(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window = main_window;
    let state_manager = sm;

    let handler = SearchPageHandler::new(&main_window, &state_manager);
    handler.initialize();

    let res = ProtoSearchResult {
        songs: vec![ProtoSong {
            song: Some(InnerSong {
                id: Some("search_song_2".to_string()),
                title: Some("Second Song".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        }],
        ..Default::default()
    };

    let cache_dir = state_manager.get_cache_dir();
    SearchPageHandler::update_provider_result(&main_window, &cache_dir, res, None, false, true);

    let results = main_window
        .global::<SearchPageProps>()
        .get_provider_results();
    assert_eq!(results.row_count(), 1);
    assert_eq!(results.row_data(0).unwrap().extension, "");
    assert_eq!(results.row_data(0).unwrap().extension_name, "");
    assert!(results.row_data(0).unwrap().loading);
    assert_eq!(results.row_data(0).unwrap().songs.row_count(), 1);

    let detail = ExtensionDetail {
        name: "Test Ext Display".to_string(),
        package_name: "test.ext.package".to_string(),
        scopes: vec![ScopePermission::new(
            ExtensionProviderScope::Search,
            "Search provider",
        )],
        ..Default::default()
    };
    SearchPageHandler::update_provider_result(
        &main_window,
        &cache_dir,
        ProtoSearchResult::default(),
        Some(detail),
        false,
        false,
    );

    let results_after = main_window
        .global::<SearchPageProps>()
        .get_provider_results();
    assert_eq!(results_after.row_count(), 2);
    assert_eq!(
        results_after.row_data(1).unwrap().extension,
        "test.ext.package"
    );
    assert_eq!(
        results_after.row_data(1).unwrap().extension_name,
        "Test Ext Display"
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_search_page_handler_auto_switches_tab(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window = main_window;
    let state_manager = sm;

    main_window
        .global::<SearchPageProps>()
        .set_provider_results(slint::ModelRc::new(slint::VecModel::default()));
    main_window.global::<SearchPageProps>().set_active_tab(0);

    let res = ProtoSearchResult {
        playlists: vec![songs_proto::moosync::types::Playlist {
            playlist_id: Some("pl_auto".to_string()),
            playlist_name: "Auto Switch Playlist".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };

    let cache_dir = state_manager.get_cache_dir();
    SearchPageHandler::update_provider_result(&main_window, &cache_dir, res, None, false, false);

    assert_eq!(main_window.global::<SearchPageProps>().get_active_tab(), 3);
}
