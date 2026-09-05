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
use slint::{ComponentHandle, Model};
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
    let handler = SearchPageHandler::new(&main_window, &sm);
    handler.initialize();

    main_window
        .global::<AppCallbacks>()
        .invoke_search_term_changed("   ".into());

    assert_eq!(
        main_window
            .global::<SearchPageProps>()
            .get_provider_results()
            .row_count(),
        0
    );
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
    let handler = SearchPageHandler::new(&main_window, &sm);
    handler.initialize();

    main_window
        .global::<AppCallbacks>()
        .invoke_search_term_changed("".into());

    assert_eq!(
        main_window
            .global::<SearchPageProps>()
            .get_provider_results()
            .row_count(),
        0
    );
}
