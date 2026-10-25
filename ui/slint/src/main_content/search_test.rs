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

use extensions_proto::moosync::types::{ExtensionDetail, ExtensionProviderScope};
use rstest::rstest;
use slint::{ComponentHandle, Model, ModelRc};
use songs_proto::moosync::types::{
    InnerSong, SearchResult as ProtoSearchResult, Song as ProtoSong,
};
use tracing_test::traced_test;

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
    let main_window = Box::leak(Box::new(main_window));
    let state_manager = Box::leak(Box::new(sm));
    let handler = SearchPageHandler::new(main_window, state_manager);
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
    let main_window = Box::leak(Box::new(main_window));
    let state_manager = Box::leak(Box::new(sm));
    let handler = SearchPageHandler::new(main_window, state_manager);
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
    let main_window = Box::leak(Box::new(main_window));
    let state_manager = Box::leak(Box::new(sm));

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
    SearchPageHandler::update_provider_result(main_window, &cache_dir, res, None, false, false);

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
    let main_window = Box::leak(Box::new(main_window));
    let state_manager = Box::leak(Box::new(sm));
    let handler = SearchPageHandler::new(main_window, state_manager);
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
    let main_window = Box::leak(Box::new(main_window));
    let state_manager = Box::leak(Box::new(sm));

    let handler = SearchPageHandler::new(main_window, state_manager);
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
    let main_window = Box::leak(Box::new(main_window));
    let state_manager = Box::leak(Box::new(sm));

    let handler = SearchPageHandler::new(main_window, state_manager);
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
    SearchPageHandler::update_provider_result(main_window, &cache_dir, res, None, false, true);

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
        scopes: vec![ExtensionProviderScope::Search as i32],
        ..Default::default()
    };
    SearchPageHandler::update_provider_result(
        main_window,
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
