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
    AllSongsPageProps, ContextMenuCallbacks, ContextMenuItem, MainWindow, SongModel,
    main_content::all_songs::AllSongsPageHandler,
    pages::PageHandler,
    test_utils::{TestSlintSmContext, main_window, state_manager_fixture},
    utils::IntoVec,
};

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_all_songs_page_handler_on_hide(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let dummy_songs = vec![SongModel::default(), SongModel::default()];
    main_window
        .global::<AllSongsPageProps>()
        .set_songs(ModelRc::new(VecModel::from(dummy_songs)));
    let handler = AllSongsPageHandler::new(&main_window, &sm);

    handler.on_hide();

    assert_eq!(
        main_window
            .global::<AllSongsPageProps>()
            .get_songs()
            .row_count(),
        0
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_all_songs_context_menu_items(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let handler = AllSongsPageHandler::new(&main_window, &sm);
    handler.initialize();

    let song_with_path = SongModel {
        path: "/path/to/song.mp3".into(),
        ..Default::default()
    };
    let models = ModelRc::new(VecModel::from(vec![song_with_path]));

    let items_rc: ModelRc<ContextMenuItem> = main_window
        .global::<ContextMenuCallbacks>()
        .invoke_get_song_menu_items(models);

    let items_vec: Vec<ContextMenuItem> = items_rc.into_vec();
    assert_eq!(items_vec.len(), 4);
    assert_eq!(items_vec[0].action_id, "play_now");
}
