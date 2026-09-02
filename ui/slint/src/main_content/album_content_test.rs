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
use songs_proto::moosync::types::{Album, InnerSong, Song};
use tracing_test::traced_test;

use crate::{
    AlbumContentPageProps, AlbumsPageProps, MainWindow, SongModel,
    main_content::album_content::AlbumContentPageHandler,
    pages::PageHandler,
    test_utils::{TestSlintSmContext, run_slint_test, wait_until},
};

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_album_content_page_handler_on_show() {
    run_slint_test(do_album_content_page_handler_on_show);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_album_content_page_handler_on_show(
    main_window: &'static MainWindow,
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

    let inserted_albums = db
        .get_entity_by_options(songs_proto::moosync::types::GetEntityOptions {
            album: Some(album),
            ..Default::default()
        })
        .unwrap();
    if let Some(songs_proto::moosync::types::entity_result::Result::Albums(list)) =
        inserted_albums.result
    {
        if let Some(first_album) = list.albums.into_iter().next() {
            main_window
                .global::<AlbumsPageProps>()
                .set_selected_album(first_album.into());
        }
    }

    let handler = AlbumContentPageHandler::new(main_window, &sm);
    handler.on_show();

    let loaded = wait_until(|| {
        main_window
            .global::<AlbumContentPageProps>()
            .get_songs()
            .row_count()
            == 1
    })
    .await;

    assert!(loaded);
    assert_eq!(
        main_window
            .global::<AlbumContentPageProps>()
            .get_songs()
            .row_count(),
        1
    );
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_album_content_page_handler_on_hide() {
    run_slint_test(do_album_content_page_handler_on_hide);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_album_content_page_handler_on_hide(
    main_window: &'static MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let dummy_songs = vec![SongModel::default(), SongModel::default()];
    main_window
        .global::<AlbumContentPageProps>()
        .set_songs(ModelRc::new(VecModel::from(dummy_songs)));
    assert_eq!(
        main_window
            .global::<AlbumContentPageProps>()
            .get_songs()
            .row_count(),
        2
    );

    let handler = AlbumContentPageHandler::new(main_window, &sm);
    handler.on_hide();

    let row_count = main_window
        .global::<AlbumContentPageProps>()
        .get_songs()
        .row_count();

    assert_eq!(row_count, 0);
}
