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

use preferences::keys::{AUTO_STARTUP, PreferenceItemExt};
use rstest::rstest;
use slint::ComponentHandle;
use tracing_test::traced_test;

use crate::{
    MainWindow,
    pages::PageHandler,
    settings::{handle_preference_change, system::SystemPageHandler},
    test_utils::{TestSlintSmContext, main_window, state_manager_fixture},
};

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_system_page_handler_initialize(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let handler = SystemPageHandler::new(&main_window, &sm);

    handler.initialize();

    assert_eq!(SystemPageHandler::get_preferences().len(), 6);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_system_page_handler_on_show(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let handler = SystemPageHandler::new(&main_window, &sm);

    handler.on_show();

    assert_eq!(SystemPageHandler::get_preferences().len(), 6);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_system_page_handler_handle_change_auto_startup(
    _main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    handle_preference_change(
        "auto_startup".to_string(),
        "".to_string(),
        true,
        0.0,
        vec![],
        sm.clone(),
        _main_window.as_weak(),
    )
    .await;

    let config = sm.get_preference_config().await;
    let auto_startup = config.load(&AUTO_STARTUP).value::<bool>().unwrap();
    assert!(auto_startup);
}
