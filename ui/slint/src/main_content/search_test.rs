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
use slint::{ComponentHandle, Model};
use songs_proto::moosync::types::{InnerSong, Song};
use tracing_test::traced_test;

use crate::{
    AppCallbacks, MainWindow, SearchPageProps,
    main_content::search::SearchPageHandler,
    pages::PageHandler,
    test_utils::{TestSlintSmContext, run_slint_test, wait_until},
};

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_search_page_handler_initialize() { run_slint_test(do_search_page_handler_initialize); }

#[tracing::instrument(level = "debug", skip_all)]
async fn do_search_page_handler_initialize(
    main_window: &'static MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let song = Song {
        song: Some(InnerSong {
            id: Some("search_test_id".into()),
            title: Some("Searchable Track".into()),
            path: Some("/music/search.mp3".into()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let db = sm.get_database().await;
    assert_ok!(db.insert_songs(vec![song]));

    let handler = SearchPageHandler::new(main_window, &sm);
    handler.initialize();

    main_window
        .global::<AppCallbacks>()
        .invoke_search_term_changed("Searchable".into());

    let loaded = wait_until(|| {
        let provider_results = main_window
            .global::<SearchPageProps>()
            .get_provider_results();
        if provider_results.row_count() == 1 {
            let local_provider = provider_results.row_data(0).unwrap();
            return local_provider.songs.row_count() == 1;
        }
        false
    })
    .await;

    assert!(loaded);
    let provider_results = main_window
        .global::<SearchPageProps>()
        .get_provider_results();
    assert_eq!(provider_results.row_count(), 1);
    let local_provider = provider_results.row_data(0).unwrap();
    assert_eq!(local_provider.songs.row_count(), 1);
    assert_eq!(
        local_provider.songs.row_data(0).unwrap().title,
        "Searchable Track"
    );
}
