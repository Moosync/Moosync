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
use songs_proto::moosync::types::{InnerSong, Playlist, Song};
use tracing_test::traced_test;

use crate::{
    ExtensionProviderItem, MainWindow, PlaylistContentPageProps, PlaylistsPageProps, SongModel,
    main_content::playlist_content::{PlaylistContentPageHandler, PlaylistSongProvider},
    pages::PageHandler,
    test_utils::{TestSlintSmContext, main_window, state_manager_fixture},
    utils::EntitySongProvider,
};

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_playlist_song_provider_extension_scope() {
    let scope = PlaylistSongProvider::extension_scope();

    assert_eq!(scope, Some(ExtensionProviderScope::PlaylistSongs));
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_playlist_song_provider_get_entity(main_window: MainWindow) {
    let playlist = Playlist {
        playlist_name: "Selected Playlist".into(),
        extension: Some("ext.pkg".into()),
        ..Default::default()
    };
    main_window
        .global::<PlaylistsPageProps>()
        .set_selected_playlist(playlist.into());

    let (ret_playlist, ext) = PlaylistSongProvider::get_entity(&main_window);

    assert_eq!(ret_playlist.playlist_name.as_str(), "Selected Playlist");
    assert_eq!(ext.as_str(), "ext.pkg");
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_playlist_song_provider_set_and_get_songs(main_window: MainWindow) {
    let dummy_songs = vec![SongModel::default(), SongModel::default()];
    let model = ModelRc::new(VecModel::from(dummy_songs));

    PlaylistSongProvider::set_songs(&main_window, model);
    let songs = PlaylistSongProvider::get_songs(&main_window);

    assert_eq!(songs.len(), 2);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_playlist_song_provider_clear_ui(main_window: MainWindow) {
    let dummy_songs = vec![SongModel::default()];
    let dummy_exts = vec![ExtensionProviderItem::default()];
    PlaylistSongProvider::set_songs(&main_window, ModelRc::new(VecModel::from(dummy_songs)));
    PlaylistSongProvider::set_extensions(&main_window, ModelRc::new(VecModel::from(dummy_exts)));

    PlaylistSongProvider::clear_ui(&main_window);

    assert_eq!(
        main_window
            .global::<PlaylistContentPageProps>()
            .get_songs()
            .row_count(),
        0
    );
    assert_eq!(
        main_window
            .global::<PlaylistContentPageProps>()
            .get_extension_providers()
            .row_count(),
        0
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_playlist_song_provider_fetch_local_songs_success(
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let song = Song {
        song: Some(InnerSong {
            id: Some("s_pl_content".into()),
            title: Some("Song In Playlist".into()),
            path: Some("/music/target_pl.mp3".into()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let playlist = Playlist {
        playlist_id: Some("pl_content_1".into()),
        playlist_name: "Target Playlist".into(),
        ..Default::default()
    };
    let db = sm.get_database().await;
    assert_ok!(db.insert_songs(vec![song.clone()]));
    assert_ok!(db.create_playlist_with_songs(playlist.clone(), &[song]));

    let songs_res = PlaylistSongProvider::fetch_local_songs(&sm, playlist).await;

    assert_ok!(&songs_res);
    let songs = songs_res.unwrap();
    assert_eq!(songs.len(), 1);
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
    let dummy_songs = vec![SongModel::default(), SongModel::default()];
    main_window
        .global::<PlaylistContentPageProps>()
        .set_songs(ModelRc::new(VecModel::from(dummy_songs)));
    let handler = PlaylistContentPageHandler::new(&main_window, &sm);

    handler.on_hide();

    assert_eq!(
        main_window
            .global::<PlaylistContentPageProps>()
            .get_songs()
            .row_count(),
        0
    );
}
