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

use extensions_proto::moosync::types::ExtensionDetail;
use rstest::rstest;
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use songs_proto::moosync::types::{InnerSong, Song};
use tracing_test::traced_test;

use crate::{
    ExplorePageProps, MainWindow, ProviderRecommendations,
    main_content::explore::ExplorePageHandler,
    pages::PageHandler,
    test_utils::{TestSlintSmContext, main_window, state_manager_fixture},
};

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_explore_page_handler_on_show_empty(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window = Box::leak(Box::new(main_window));
    let state_manager = Box::leak(Box::new(sm));
    let handler = ExplorePageHandler::new(main_window, state_manager);

    handler.on_show();
    let count = main_window
        .global::<ExplorePageProps>()
        .get_provider_recommendations()
        .row_count();

    assert_eq!(count, 0);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_explore_page_handler_on_hide(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window = Box::leak(Box::new(main_window));
    let state_manager = Box::leak(Box::new(sm));
    let dummy_recs = vec![
        ProviderRecommendations::default(),
        ProviderRecommendations::default(),
    ];
    main_window
        .global::<ExplorePageProps>()
        .set_provider_recommendations(ModelRc::new(VecModel::from(dummy_recs)));
    assert_eq!(
        main_window
            .global::<ExplorePageProps>()
            .get_provider_recommendations()
            .row_count(),
        2
    );

    let handler = ExplorePageHandler::new(main_window, state_manager);
    handler.on_hide();

    let count = main_window
        .global::<ExplorePageProps>()
        .get_provider_recommendations()
        .row_count();

    assert_eq!(count, 0);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_explore_page_handler_render_recommendations(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let detail = ExtensionDetail {
        name: "Test Provider".to_string(),
        package_name: "test.provider".to_string(),
        ..Default::default()
    };
    let song = Song {
        song: Some(InnerSong {
            id: Some("rec_song_1".to_string()),
            title: Some("Rec Song".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let recommendations = vec![(detail, None, vec![song])];

    ExplorePageHandler::render_recommendations(&main_window, &sm, recommendations);

    let props = main_window.global::<ExplorePageProps>();
    let recs = props.get_provider_recommendations();
    assert_eq!(recs.row_count(), 1);
    assert_eq!(
        recs.row_data(0).unwrap().provider_name.as_str(),
        "Test Provider"
    );
    assert_eq!(recs.row_data(0).unwrap().songs.row_count(), 1);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_explore_page_handler_fetch_all_recommendations_empty(
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;

    let res = ExplorePageHandler::fetch_all_recommendations(&sm).await;

    assert!(res.is_ok());
    assert!(res.unwrap().is_empty());
}
