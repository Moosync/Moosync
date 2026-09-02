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

use assertables::assert_ok;
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use songs_proto::moosync::types::Playlist;
use tracing_test::traced_test;

use crate::{
    MainWindow, PlaylistModel, PlaylistsPageProps,
    main_content::playlists::PlaylistsPageHandler,
    pages::PageHandler,
    test_utils::{TestSlintSmContext, run_slint_test, wait_until},
};

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_playlists_page_handler_on_show() { run_slint_test(do_playlists_page_handler_on_show); }

#[tracing::instrument(level = "debug", skip_all)]
async fn do_playlists_page_handler_on_show(
    main_window: &'static MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let playlist = Playlist {
        playlist_name: "My Test Playlist".into(),
        ..Default::default()
    };
    let db = sm.get_database().await;
    assert_ok!(db.create_playlist_with_songs(playlist, &[]));

    let handler = PlaylistsPageHandler::new(main_window, &sm);
    handler.on_show();

    let loaded = wait_until(|| {
        main_window
            .global::<PlaylistsPageProps>()
            .get_playlists()
            .row_count()
            == 1
    })
    .await;

    assert!(loaded);
    assert_eq!(
        main_window
            .global::<PlaylistsPageProps>()
            .get_playlists()
            .row_count(),
        1
    );
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_playlists_page_handler_on_hide() { run_slint_test(do_playlists_page_handler_on_hide); }

#[tracing::instrument(level = "debug", skip_all)]
async fn do_playlists_page_handler_on_hide(
    main_window: &'static MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let dummy_playlists = vec![PlaylistModel::default(), PlaylistModel::default()];
    main_window
        .global::<PlaylistsPageProps>()
        .set_playlists(ModelRc::new(VecModel::from(dummy_playlists)));
    assert_eq!(
        main_window
            .global::<PlaylistsPageProps>()
            .get_playlists()
            .row_count(),
        2
    );

    let handler = PlaylistsPageHandler::new(main_window, &sm);
    handler.on_hide();

    let row_count = main_window
        .global::<PlaylistsPageProps>()
        .get_playlists()
        .row_count();

    assert_eq!(row_count, 0);
}
