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

use rstest::rstest;
use slint::{ComponentHandle, Model, ModelRc, VecModel};
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
    let handler = ExplorePageHandler::new(&main_window, &sm);

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

    let handler = ExplorePageHandler::new(&main_window, &sm);
    handler.on_hide();

    let count = main_window
        .global::<ExplorePageProps>()
        .get_provider_recommendations()
        .row_count();

    assert_eq!(count, 0);
}
