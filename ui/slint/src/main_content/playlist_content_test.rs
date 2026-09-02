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
use slint::{ComponentHandle, Model, ModelRc};
use tracing_test::traced_test;

use crate::{
    MainWindow, PlaylistContentPageProps,
    main_content::playlist_content::PlaylistContentPageHandler,
    pages::PageHandler,
    test_utils::{TestSlintSmContext, main_window, state_manager_fixture},
};

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_playlist_content_page_handler_on_show(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    main_window
        .global::<PlaylistContentPageProps>()
        .set_songs(ModelRc::default());
    let handler = PlaylistContentPageHandler::new(&main_window, &sm);

    handler.on_show();
    let row_count = main_window
        .global::<PlaylistContentPageProps>()
        .get_songs()
        .row_count();

    assert_eq!(row_count, 0);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_playlist_content_page_handler_on_hide(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let handler = PlaylistContentPageHandler::new(&main_window, &sm);

    handler.on_hide();
    let row_count = main_window
        .global::<PlaylistContentPageProps>()
        .get_songs()
        .row_count();

    assert_eq!(row_count, 0);
}
