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
use extensions_proto::moosync::types::ExtensionProviderScope;
use rstest::rstest;
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use songs_proto::moosync::types::{Album, InnerSong, Song};
use tracing_test::traced_test;

use crate::{
    AlbumContentPageProps, AlbumsPageProps, ExtensionProviderItem, MainWindow, SongModel,
    main_content::album_content::{AlbumContentPageHandler, AlbumSongProvider},
    pages::PageHandler,
    test_utils::{TestSlintSmContext, main_window, state_manager_fixture},
    utils::EntitySongProvider,
};

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_album_song_provider_extension_scope() {
    let scope = AlbumSongProvider::extension_scope();

    assert_eq!(scope, Some(ExtensionProviderScope::AlbumSongs));
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_album_song_provider_get_entity(main_window: MainWindow) {
    let album = Album {
        album_name: Some("Selected Album".into()),
        extension: Some("ext.pkg".into()),
        ..Default::default()
    };
    main_window
        .global::<AlbumsPageProps>()
        .set_selected_album(album.into());

    let (ret_album, ext) = AlbumSongProvider::get_entity(&main_window);

    assert_eq!(ret_album.album_name.as_deref(), Some("Selected Album"));
    assert_eq!(ext.as_str(), "ext.pkg");
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_album_song_provider_set_and_get_songs(main_window: MainWindow) {
    let dummy_songs = vec![SongModel::default(), SongModel::default()];
    let model = ModelRc::new(VecModel::from(dummy_songs));

    AlbumSongProvider::set_songs(&main_window, model);
    let songs = AlbumSongProvider::get_songs(&main_window);

    assert_eq!(songs.len(), 2);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_album_song_provider_clear_ui(main_window: MainWindow) {
    let dummy_songs = vec![SongModel::default()];
    let dummy_exts = vec![ExtensionProviderItem::default()];
    AlbumSongProvider::set_songs(&main_window, ModelRc::new(VecModel::from(dummy_songs)));
    AlbumSongProvider::set_extensions(&main_window, ModelRc::new(VecModel::from(dummy_exts)));

    AlbumSongProvider::clear_ui(&main_window);

    assert_eq!(
        main_window
            .global::<AlbumContentPageProps>()
            .get_songs()
            .row_count(),
        0
    );
    assert_eq!(
        main_window
            .global::<AlbumContentPageProps>()
            .get_extension_providers()
            .row_count(),
        0
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_album_song_provider_fetch_local_songs_success(
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let album = Album {
        album_name: Some("Target Album".into()),
        ..Default::default()
    };
    let song = Song {
        song: Some(InnerSong {
            id: Some("s_alb_content".into()),
            title: Some("Song In Target Album".into()),
            path: Some("/music/target_alb.mp3".into()),
            ..Default::default()
        }),
        album: Some(album.clone()),
        ..Default::default()
    };
    let db = sm.get_database().await;
    assert_ok!(db.insert_songs(vec![song]));

    let songs_res = AlbumSongProvider::fetch_local_songs(&sm, album).await;

    assert_ok!(&songs_res);
    let songs = songs_res.unwrap();
    assert_eq!(songs.len(), 1);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_album_content_page_handler_on_hide(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let dummy_songs = vec![SongModel::default(), SongModel::default()];
    main_window
        .global::<AlbumContentPageProps>()
        .set_songs(ModelRc::new(VecModel::from(dummy_songs)));
    let handler = AlbumContentPageHandler::new(&main_window, &sm);

    handler.on_hide();

    assert_eq!(
        main_window
            .global::<AlbumContentPageProps>()
            .get_songs()
            .row_count(),
        0
    );
}
