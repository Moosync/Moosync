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
use songs_proto::moosync::types::{Album, InnerSong, Song};
use tracing_test::traced_test;

use crate::{
    AlbumModel, AlbumsPageProps, MainWindow,
    main_content::albums::{AlbumListProvider, AlbumsPageHandler},
    pages::PageHandler,
    test_utils::{TestSlintSmContext, main_window, state_manager_fixture},
    utils::EntityListProvider,
};

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_album_list_provider_name() {
    let name = AlbumListProvider::name();

    assert_eq!(name, "Albums");
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_album_list_provider_to_model() {
    let album = Album {
        album_name: Some("Test Album".into()),
        album_artist: Some("Test Artist".into()),
        ..Default::default()
    };

    let model = AlbumListProvider::to_model(album);

    assert_eq!(model.title.as_str(), "Test Album");
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_album_list_provider_set_models(main_window: MainWindow) {
    let dummy_albums = vec![AlbumModel::default(), AlbumModel::default()];
    let model = ModelRc::new(VecModel::from(dummy_albums));

    AlbumListProvider::set_models(&main_window, model);

    assert_eq!(
        main_window
            .global::<AlbumsPageProps>()
            .get_albums()
            .row_count(),
        2
    );
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_album_list_provider_clear_models(main_window: MainWindow) {
    let dummy_albums = vec![AlbumModel::default(), AlbumModel::default()];
    let model = ModelRc::new(VecModel::from(dummy_albums));
    AlbumListProvider::set_models(&main_window, model);

    AlbumListProvider::clear_models(&main_window);

    assert_eq!(
        main_window
            .global::<AlbumsPageProps>()
            .get_albums()
            .row_count(),
        0
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_album_list_provider_fetch_entities_success(
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let song = Song {
        song: Some(InnerSong {
            id: Some("s_alb".into()),
            title: Some("Album Song".into()),
            path: Some("/music/alb.mp3".into()),
            ..Default::default()
        }),
        album: Some(Album {
            album_name: Some("Album 1".into()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let db = sm.get_database().await;
    assert_ok!(db.insert_songs(vec![song]));

    let entities_res = AlbumListProvider::fetch_entities(&sm).await;

    assert_ok!(&entities_res);
    let entities = entities_res.unwrap();
    assert_eq!(entities.len(), 1);
    assert_eq!(entities[0].album_name.as_deref(), Some("Album 1"));
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_albums_page_handler_on_hide(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let dummy_albums = vec![AlbumModel::default(), AlbumModel::default()];
    main_window
        .global::<AlbumsPageProps>()
        .set_albums(ModelRc::new(VecModel::from(dummy_albums)));
    let handler = AlbumsPageHandler::new(&main_window, &sm);

    handler.on_hide();

    assert_eq!(
        main_window
            .global::<AlbumsPageProps>()
            .get_albums()
            .row_count(),
        0
    );
}
