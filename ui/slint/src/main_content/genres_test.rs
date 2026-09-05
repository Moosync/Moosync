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
    GenreModel, GenresPageProps, MainWindow,
    main_content::genres::{GenreListProvider, GenresPageHandler},
    pages::PageHandler,
    test_utils::{TestSlintSmContext, main_window, state_manager_fixture},
    utils::EntityListProvider,
};

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_genre_list_provider_name() {
    let name = GenreListProvider::name();

    assert_eq!(name, "Genres");
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_genre_list_provider_to_model() {
    let genre = Genre {
        genre_name: Some("Rock".into()),
        ..Default::default()
    };

    let model = GenreListProvider::to_model(genre);

    assert_eq!(model.title.as_str(), "Rock");
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_genre_list_provider_set_models(main_window: MainWindow) {
    let dummy_genres = vec![GenreModel::default(), GenreModel::default()];
    let model = ModelRc::new(VecModel::from(dummy_genres));

    GenreListProvider::set_models(&main_window, model);

    assert_eq!(
        main_window
            .global::<GenresPageProps>()
            .get_genres()
            .row_count(),
        2
    );
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_genre_list_provider_clear_models(main_window: MainWindow) {
    let dummy_genres = vec![GenreModel::default(), GenreModel::default()];
    let model = ModelRc::new(VecModel::from(dummy_genres));
    GenreListProvider::set_models(&main_window, model);

    GenreListProvider::clear_models(&main_window);

    assert_eq!(
        main_window
            .global::<GenresPageProps>()
            .get_genres()
            .row_count(),
        0
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_genre_list_provider_fetch_entities_success(
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let song = Song {
        song: Some(InnerSong {
            id: Some("s_gen".into()),
            title: Some("Genre Song".into()),
            path: Some("/music/gen.mp3".into()),
            ..Default::default()
        }),
        genre: vec![Genre {
            genre_name: Some("Rock".into()),
            ..Default::default()
        }],
        ..Default::default()
    };
    let db = sm.get_database().await;
    assert_ok!(db.insert_songs(vec![song]));

    let entities_res = GenreListProvider::fetch_entities(&sm).await;

    assert_ok!(&entities_res);
    let entities = entities_res.unwrap();
    assert_eq!(entities.len(), 1);
    assert_eq!(entities[0].genre_name.as_deref(), Some("Rock"));
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_genres_page_handler_on_hide(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let dummy_genres = vec![GenreModel::default(), GenreModel::default()];
    main_window
        .global::<GenresPageProps>()
        .set_genres(ModelRc::new(VecModel::from(dummy_genres)));
    let handler = GenresPageHandler::new(&main_window, &sm);

    handler.on_hide();

    assert_eq!(
        main_window
            .global::<GenresPageProps>()
            .get_genres()
            .row_count(),
        0
    );
}
