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
use rstest::rstest;
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use songs_proto::moosync::types::Playlist;
use tracing_test::traced_test;

use crate::{
    MainWindow, PlaylistModel, PlaylistsPageProps,
    main_content::playlists::{PlaylistListProvider, PlaylistsPageHandler},
    pages::PageHandler,
    test_utils::{TestSlintSmContext, main_window, state_manager_fixture},
    utils::EntityListProvider,
};

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_playlist_list_provider_name() {
    let name = PlaylistListProvider::name();

    assert_eq!(name, "Playlists");
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_playlist_list_provider_to_model() {
    let playlist = Playlist {
        playlist_name: "My Test Playlist".into(),
        ..Default::default()
    };

    let model = PlaylistListProvider::to_model(playlist);

    assert_eq!(model.title.as_str(), "My Test Playlist");
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_playlist_list_provider_set_models(main_window: MainWindow) {
    let dummy_playlists = vec![PlaylistModel::default(), PlaylistModel::default()];
    let model = ModelRc::new(VecModel::from(dummy_playlists));

    PlaylistListProvider::set_models(&main_window, model);

    assert_eq!(
        main_window
            .global::<PlaylistsPageProps>()
            .get_playlists()
            .row_count(),
        2
    );
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_playlist_list_provider_clear_models(main_window: MainWindow) {
    let dummy_playlists = vec![PlaylistModel::default(), PlaylistModel::default()];
    let model = ModelRc::new(VecModel::from(dummy_playlists));
    PlaylistListProvider::set_models(&main_window, model);

    PlaylistListProvider::clear_models(&main_window);

    assert_eq!(
        main_window
            .global::<PlaylistsPageProps>()
            .get_playlists()
            .row_count(),
        0
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_playlist_list_provider_fetch_entities_success(
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let playlist = Playlist {
        playlist_name: "My Test Playlist".into(),
        ..Default::default()
    };
    let db = sm.get_database().await;
    assert_ok!(db.create_playlist_with_songs(playlist, &[]));

    let entities_res = PlaylistListProvider::fetch_entities(&sm).await;

    assert_ok!(&entities_res);
    let entities = entities_res.unwrap();
    assert_eq!(entities.len(), 1);
    assert_eq!(entities[0].playlist_name.as_str(), "My Test Playlist");
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_playlists_page_handler_on_hide(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let dummy_playlists = vec![PlaylistModel::default(), PlaylistModel::default()];
    main_window
        .global::<PlaylistsPageProps>()
        .set_playlists(ModelRc::new(VecModel::from(dummy_playlists)));
    let handler = PlaylistsPageHandler::new(&main_window, &sm);

    handler.on_hide();

    assert_eq!(
        main_window
            .global::<PlaylistsPageProps>()
            .get_playlists()
            .row_count(),
        0
    );
}
