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

use preferences::keys::{MUSIC_PATHS, PreferenceItemExt};
use rstest::rstest;
use slint::ComponentHandle;
use tracing_test::traced_test;

use crate::{
    MainWindow,
    pages::PageHandler,
    settings::{handle_preference_change, paths::PathsPageHandler},
    test_utils::{TestSlintSmContext, main_window, state_manager_fixture},
};

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_paths_page_handler_initialize(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let handler = PathsPageHandler::new(&main_window, &sm);

    handler.initialize();

    assert_eq!(PathsPageHandler::get_preferences().len(), 7);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_paths_page_handler_on_show(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let handler = PathsPageHandler::new(&main_window, &sm);

    handler.on_show();

    assert_eq!(PathsPageHandler::get_preferences().len(), 7);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_paths_page_handler_handle_change_music_paths(
    _main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    handle_preference_change(
        "music_paths".to_string(),
        "".to_string(),
        false,
        0.0,
        vec!["/test/path".to_string()],
        sm.clone(),
        _main_window.as_weak(),
    )
    .await;

    let config = sm.get_preference_config().await;
    let paths = config.load(&MUSIC_PATHS).value::<Vec<String>>().unwrap();
    assert!(paths.contains(&"/test/path".to_string()));
}
