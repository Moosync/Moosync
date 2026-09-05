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
use songs_proto::moosync::types::{Genre, InnerSong, Song};
use tracing_test::traced_test;

use crate::{
    GenreContentPageProps, GenresPageProps, MainWindow, SongModel,
    main_content::genre_content::{GenreContentPageHandler, GenreSongProvider},
    pages::PageHandler,
    test_utils::{TestSlintSmContext, main_window, state_manager_fixture},
    utils::EntitySongProvider,
};
#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_genre_song_provider_get_entity(main_window: MainWindow) {
    let genre = Genre {
        genre_name: Some("Selected Genre".into()),
        ..Default::default()
    };
    main_window
        .global::<GenresPageProps>()
        .set_selected_genre(genre.into());

    let (ret_genre, ext) = GenreSongProvider::get_entity(&main_window);

    assert_eq!(ret_genre.genre_name.as_deref(), Some("Selected Genre"));
    assert!(ext.is_empty());
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_genre_song_provider_clear_ui(main_window: MainWindow) {
    let dummy_songs = vec![SongModel::default()];
    GenreSongProvider::set_songs(&main_window, ModelRc::new(VecModel::from(dummy_songs)));

    GenreSongProvider::clear_ui(&main_window);

    assert_eq!(
        main_window
            .global::<GenreContentPageProps>()
            .get_songs()
            .row_count(),
        0
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_genre_song_provider_fetch_local_songs_success(
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let genre = Genre {
        genre_name: Some("Target Genre".into()),
        ..Default::default()
    };
    let song = Song {
        song: Some(InnerSong {
            id: Some("s_gen_content".into()),
            title: Some("Song In Target Genre".into()),
            path: Some("/music/target_gen.mp3".into()),
            ..Default::default()
        }),
        genre: vec![genre.clone()],
        ..Default::default()
    };
    let db = sm.get_database().await;
    assert_ok!(db.insert_songs(vec![song]));

    let songs_res = GenreSongProvider::fetch_local_songs(&sm, genre).await;

    assert_ok!(&songs_res);
    let songs = songs_res.unwrap();
    assert_eq!(songs.len(), 1);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_genre_content_page_handler_on_hide(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let dummy_songs = vec![SongModel::default(), SongModel::default()];
    main_window
        .global::<GenreContentPageProps>()
        .set_songs(ModelRc::new(VecModel::from(dummy_songs)));
    let handler = GenreContentPageHandler::new(&main_window, &sm);

    handler.on_hide();

    assert_eq!(
        main_window
            .global::<GenreContentPageProps>()
            .get_songs()
            .row_count(),
        0
    );
}
