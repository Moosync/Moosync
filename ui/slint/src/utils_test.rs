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

use extensions_proto::moosync::types::{ExtensionDetail, FetchedExtensionManifest};
use slint::{Model, ModelRc, VecModel};
use songs_proto::moosync::types::{
    Album, Artist, Genre, GetEntityOptions, InnerSong, Playlist, SearchResult as ProtoSearchResult,
    Song, entity_result,
};
use state_manager::StateManager;
use tempdir::TempDir;
use types::plugin::PluginContext;

use crate::{
    SongSortCriterion,
    utils::{
        LazySongVecModel, cache_image, default_empty_icon, default_entity_cover,
        default_folder_icon, default_song_cover, filter_and_sort_songs, get_safe_name, parse_color,
        parse_length, save_queue, song_model_to_song, to_album_model, to_artist_model,
        to_extension_item, to_fetched_extension_item, to_genre_model, to_playlist_model,
        to_search_result, to_song_model,
    },
};

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_get_safe_name_replaces_special_chars() {
    let result = get_safe_name("https://example.com/cover.jpg");

    assert_eq!(result, "https___example_com_cover_jpg");
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_get_safe_name_replaces_hyphens() {
    let result = get_safe_name("abc-123_xyz");

    assert_eq!(result, "abc_123_xyz");
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_song_model() {
    let song = Song {
        song: Some(InnerSong {
            id: Some("id123".to_string()),
            title: Some("Song Title".to_string()),
            path: Some("/music/test.mp3".to_string()),
            duration: Some(songs_proto::duration_proto::google::protobuf::Duration {
                seconds: 240,
                nanos: 0,
            }),
            ..Default::default()
        }),
        album: Some(Album {
            album_id: Some("alb1".to_string()),
            album_name: Some("Album Name".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let model = to_song_model(&song, None);

    assert_eq!(model.id, "id123");
    assert_eq!(model.title, "Song Title");
    assert_eq!(model.album_name, "Album Name");
    assert_eq!(model.coverPathHigh.size().width, 0);
    assert_eq!(model.coverPathLow.size().width, 0);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_song_model_to_song() {
    let original = Song {
        song: Some(InnerSong {
            id: Some("id123".to_string()),
            title: Some("Song Title".to_string()),
            path: Some("/music/test.mp3".to_string()),
            ..Default::default()
        }),
        album: Some(Album {
            album_id: Some("alb1".to_string()),
            album_name: Some("Album Name".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let model = to_song_model(&original, None);

    let reconstructed = song_model_to_song(&model);

    assert_eq!(
        reconstructed.song.as_ref().unwrap().id.as_deref(),
        Some("id123")
    );
    assert_eq!(
        reconstructed.song.as_ref().unwrap().title.as_deref(),
        Some("Song Title")
    );
    assert_eq!(
        reconstructed.album.as_ref().unwrap().album_name.as_deref(),
        Some("Album Name")
    );
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_album_model() {
    let album = Album {
        album_id: Some("alb123".to_string()),
        album_name: Some("Greatest Hits".to_string()),
        album_song_count: 12.0,
        ..Default::default()
    };

    let model = to_album_model(&album, None);

    assert_eq!(model.id, "alb123");
    assert_eq!(model.title, "Greatest Hits");
    assert_eq!(model.songs_count, 12);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_artist_model() {
    let artist = Artist {
        artist_id: Some("art123".to_string()),
        artist_name: Some("Queen".to_string()),
        artist_song_count: 50.0,
        ..Default::default()
    };

    let model = to_artist_model(&artist, None);

    assert_eq!(model.id, "art123");
    assert_eq!(model.title, "Queen");
    assert_eq!(model.songs_count, 50);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_album_model_without_cover_has_placeholder() {
    let album = Album {
        album_id: Some("alb123".to_string()),
        album_name: Some("Greatest Hits".to_string()),
        album_coverpath_high: None,
        ..Default::default()
    };
    let expected = default_entity_cover();

    let model = to_album_model(&album, None);

    assert_eq!(model.id, "alb123");
    assert_eq!(model.coverPath.size(), expected.size());
    assert_ne!(model.coverPath.size().width, 0);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_artist_model_without_cover_has_placeholder() {
    let artist = Artist {
        artist_id: Some("art123".to_string()),
        artist_name: Some("Queen".to_string()),
        artist_coverpath: None,
        ..Default::default()
    };
    let expected = default_entity_cover();

    let model = to_artist_model(&artist, None);

    assert_eq!(model.id, "art123");
    assert_eq!(model.coverPath.size(), expected.size());
    assert_ne!(model.coverPath.size().width, 0);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_genre_model() {
    let genre = Genre {
        genre_id: Some("gen123".to_string()),
        genre_name: Some("Rock".to_string()),
        genre_song_count: 15.0,
    };
    let expected = default_entity_cover();

    let model = to_genre_model(&genre);

    assert_eq!(model.id, "gen123");
    assert_eq!(model.title, "Rock");
    assert_eq!(model.songs_count, 15);
    assert_eq!(model.coverPath.size(), expected.size());
    assert_ne!(model.coverPath.size().width, 0);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_playlist_model() {
    let playlist = Playlist {
        playlist_id: Some("pl123".to_string()),
        playlist_name: "Favorites".to_string(),
        playlist_song_count: 25.0,
        ..Default::default()
    };
    let expected = default_entity_cover();

    let model = to_playlist_model(&playlist, None);

    assert_eq!(model.id, "pl123");
    assert_eq!(model.title, "Favorites");
    assert_eq!(model.songs_count, 25);
    assert_eq!(model.coverPath.size(), expected.size());
    assert_ne!(model.coverPath.size().width, 0);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_extension_item() {
    let detail = ExtensionDetail {
        package_name: "com.test.ext".to_string(),
        name: "Test Extension".to_string(),
        desc: Some("A test extension".to_string()),
        version: "1.0.0".to_string(),
        author: Some("Author".to_string()),
        ..Default::default()
    };

    let item = to_extension_item(&detail);

    assert_eq!(item.package_name, "com.test.ext");
    assert_eq!(item.name, "Test Extension");
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_fetched_extension_item() {
    let manifest = FetchedExtensionManifest {
        package_name: "com.fetched.ext".to_string(),
        name: "Fetched Ext".to_string(),
        description: Some("Fetched description".to_string()),
        version: "2.0.0".to_string(),
        ..Default::default()
    };

    let fetched_item = to_fetched_extension_item(&manifest);

    assert_eq!(fetched_item.package_name, "com.fetched.ext");
    assert_eq!(fetched_item.name, "Fetched Ext");
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_search_result() {
    crate::test_utils::run_test(|| {
        use slint::ComponentHandle;
        let main_window = crate::MainWindow::new().unwrap();
        let theme = main_window.global::<crate::Theme>();
        let proto_res = ProtoSearchResult {
            songs: vec![Song {
                song: Some(InnerSong {
                    id: Some("s1".to_string()),
                    title: Some("Song 1".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }],
            albums: vec![Album {
                album_id: Some("a1".to_string()),
                album_name: Some("Album 1".to_string()),
                ..Default::default()
            }],
            artists: vec![Artist {
                artist_id: Some("ar1".to_string()),
                artist_name: Some("Artist 1".to_string()),
                ..Default::default()
            }],
            playlists: vec![Playlist {
                playlist_id: Some("p1".to_string()),
                playlist_name: "Playlist 1".to_string(),
                ..Default::default()
            }],
            genres: vec![Genre {
                genre_id: Some("g1".to_string()),
                genre_name: Some("Genre 1".to_string()),
                ..Default::default()
            }],
        };
        let tmp = TempDir::new("moosync_search_utils_test").unwrap();

        let result = to_search_result(proto_res, None, default_empty_icon(), &theme, tmp.path());

        assert_eq!(result.extension, "");
        assert_eq!(result.songs.row_count(), 1);
        assert_eq!(result.albums.row_count(), 1);
        assert_eq!(result.artists.row_count(), 1);
        assert_eq!(result.playlists.row_count(), 1);
        assert_eq!(result.genres.row_count(), 1);
    });
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_parse_color_valid() {
    let col = parse_color("#ff5733");

    assert!(col.is_some());
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_parse_color_invalid() {
    let col = parse_color("not-a-color");

    assert!(col.is_none());
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_parse_length_px() {
    let len = parse_length("16px");

    assert_eq!(len, Some(16.0));
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_parse_length_raw_number() {
    let len = parse_length("32");

    assert_eq!(len, Some(32.0));
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_parse_length_invalid() {
    let len = parse_length("invalid");

    assert!(len.is_none());
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_cache_image_nonexistent_local() {
    let tmp = TempDir::new("moosync_cache_img_test").unwrap();

    let res = cache_image("/non/existent/path.png", tmp.path()).await;

    assert!(res.is_none());
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_cache_image_existing_local() {
    let tmp = TempDir::new("moosync_cache_img_test2").unwrap();
    let local_file = tmp.path().join("test.txt");
    std::fs::write(&local_file, b"test").unwrap();

    let res = cache_image(local_file.to_str().unwrap(), tmp.path()).await;

    assert_eq!(res, Some(local_file));
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_default_song_cover() {
    let cover = default_song_cover();

    assert_ne!(cover.size().width, 0);
    assert_ne!(cover.size().height, 0);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_default_empty_icon() {
    let icon = default_empty_icon();

    assert_ne!(icon.size().width, 0);
    assert_ne!(icon.size().height, 0);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_default_folder_icon() {
    let icon = default_folder_icon();

    assert_ne!(icon.size().width, 0);
    assert_ne!(icon.size().height, 0);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_lazy_song_vec_model_row_count_and_data() {
    let tmp = TempDir::new("moosync_lazy_model_test").unwrap();
    let album = to_album_model(
        &Album {
            album_id: Some("a1".to_string()),
            album_name: Some("Album 1".to_string()),
            ..Default::default()
        },
        None,
    );
    let lazy_model = LazySongVecModel::new(vec![album], 100, 100, tmp.path().to_path_buf());

    assert_eq!(lazy_model.row_count(), 1);
    let item = lazy_model.row_data(0);
    assert!(item.is_some());
    assert_eq!(item.unwrap().title, "Album 1");
}

#[tracing::instrument(level = "debug", skip_all)]
fn create_test_song_models() -> ModelRc<crate::SongModel> {
    let song1 = to_song_model(
        &Song {
            song: Some(InnerSong {
                id: Some("1".to_string()),
                title: Some("Bravo Song".to_string()),
                year: Some("2021".to_string()),
                track_no: Some(2.0),
                ..Default::default()
            }),
            album: Some(Album {
                album_name: Some("Zulu Album".to_string()),
                ..Default::default()
            }),
            artists: vec![Artist {
                artist_name: Some("Charlie Artist".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        },
        None,
    );

    let song2 = to_song_model(
        &Song {
            song: Some(InnerSong {
                id: Some("2".to_string()),
                title: Some("Alpha Song".to_string()),
                year: Some("2023".to_string()),
                track_no: Some(1.0),
                ..Default::default()
            }),
            album: Some(Album {
                album_name: Some("Alpha Album".to_string()),
                ..Default::default()
            }),
            artists: vec![Artist {
                artist_name: Some("Delta Artist".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        },
        None,
    );

    let song3 = to_song_model(
        &Song {
            song: Some(InnerSong {
                id: Some("3".to_string()),
                title: Some("Charlie Song".to_string()),
                year: Some("2020".to_string()),
                track_no: Some(3.0),
                ..Default::default()
            }),
            album: Some(Album {
                album_name: Some("Echo Album".to_string()),
                ..Default::default()
            }),
            artists: vec![Artist {
                artist_name: Some("Alpha Artist".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        },
        None,
    );

    ModelRc::new(VecModel::from(vec![song1, song2, song3]))
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_filter_and_sort_songs_filter_by_title() {
    let tmp = TempDir::new("test_filter_title").unwrap();
    let songs = create_test_song_models();

    let result = filter_and_sort_songs(
        songs,
        "Bravo",
        SongSortCriterion::Title,
        true,
        100,
        100,
        tmp.path().to_path_buf(),
    );

    assert_eq!(result.row_count(), 1);
    assert_eq!(result.row_data(0).unwrap().title, "Bravo Song");
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_filter_and_sort_songs_search_matches_only_title() {
    let tmp = TempDir::new("test_search_title_only").unwrap();
    let songs = create_test_song_models();

    // Matching title succeeds
    let result_title = filter_and_sort_songs(
        songs.clone(),
        "Alpha",
        SongSortCriterion::Title,
        true,
        100,
        100,
        tmp.path().to_path_buf(),
    );
    assert_eq!(result_title.row_count(), 1);
    assert_eq!(result_title.row_data(0).unwrap().title, "Alpha Song");

    // Searching by artist name does not match
    let result_artist = filter_and_sort_songs(
        songs.clone(),
        "Delta",
        SongSortCriterion::Title,
        true,
        100,
        100,
        tmp.path().to_path_buf(),
    );
    assert_eq!(result_artist.row_count(), 0);

    // Searching by album name does not match
    let result_album = filter_and_sort_songs(
        songs,
        "Echo",
        SongSortCriterion::Title,
        true,
        100,
        100,
        tmp.path().to_path_buf(),
    );
    assert_eq!(result_album.row_count(), 0);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_filter_and_sort_songs_sort_by_title() {
    let tmp = TempDir::new("test_sort_title").unwrap();
    let songs = create_test_song_models();

    let result = filter_and_sort_songs(
        songs,
        "Song",
        SongSortCriterion::Title,
        true,
        100,
        100,
        tmp.path().to_path_buf(),
    );

    assert_eq!(result.row_count(), 3);
    assert_eq!(result.row_data(0).unwrap().title, "Alpha Song");
    assert_eq!(result.row_data(1).unwrap().title, "Bravo Song");
    assert_eq!(result.row_data(2).unwrap().title, "Charlie Song");
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_filter_and_sort_songs_sort_by_title_descending() {
    let tmp = TempDir::new("test_sort_title_desc").unwrap();
    let songs = create_test_song_models();

    let result = filter_and_sort_songs(
        songs,
        "",
        SongSortCriterion::Title,
        false,
        100,
        100,
        tmp.path().to_path_buf(),
    );

    assert_eq!(result.row_count(), 3);
    assert_eq!(result.row_data(0).unwrap().title, "Charlie Song");
    assert_eq!(result.row_data(1).unwrap().title, "Bravo Song");
    assert_eq!(result.row_data(2).unwrap().title, "Alpha Song");
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_filter_and_sort_songs_sort_by_date() {
    let tmp = TempDir::new("test_sort_date").unwrap();
    let songs = create_test_song_models();

    let result = filter_and_sort_songs(
        songs,
        "",
        SongSortCriterion::Date,
        true,
        100,
        100,
        tmp.path().to_path_buf(),
    );

    assert_eq!(result.row_count(), 3);
    assert_eq!(result.row_data(0).unwrap().title, "Charlie Song"); // 2020
    assert_eq!(result.row_data(1).unwrap().title, "Bravo Song"); // 2021
    assert_eq!(result.row_data(2).unwrap().title, "Alpha Song"); // 2023
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_filter_and_sort_songs_sort_by_album() {
    let tmp = TempDir::new("test_sort_album").unwrap();
    let songs = create_test_song_models();

    let result = filter_and_sort_songs(
        songs,
        "",
        SongSortCriterion::Album,
        true,
        100,
        100,
        tmp.path().to_path_buf(),
    );

    assert_eq!(result.row_count(), 3);
    assert_eq!(result.row_data(0).unwrap().album_name, "Alpha Album");
    assert_eq!(result.row_data(1).unwrap().album_name, "Echo Album");
    assert_eq!(result.row_data(2).unwrap().album_name, "Zulu Album");
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_filter_and_sort_songs_sort_by_track_number() {
    let tmp = TempDir::new("test_sort_track").unwrap();
    let songs = create_test_song_models();

    let result = filter_and_sort_songs(
        songs,
        "",
        SongSortCriterion::TrackNumber,
        true,
        100,
        100,
        tmp.path().to_path_buf(),
    );

    assert_eq!(result.row_count(), 3);
    assert_eq!(result.row_data(0).unwrap().track_no, 1.0);
    assert_eq!(result.row_data(1).unwrap().track_no, 2.0);
    assert_eq!(result.row_data(2).unwrap().track_no, 3.0);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_filter_and_sort_songs_sort_by_artist() {
    let tmp = TempDir::new("test_sort_artist").unwrap();
    let songs = create_test_song_models();

    let result = filter_and_sort_songs(
        songs,
        "",
        SongSortCriterion::Artist,
        true,
        100,
        100,
        tmp.path().to_path_buf(),
    );

    assert_eq!(result.row_count(), 3);
    assert_eq!(result.row_data(0).unwrap().title, "Charlie Song"); // Alpha Artist
    assert_eq!(result.row_data(1).unwrap().title, "Bravo Song"); // Charlie Artist
    assert_eq!(result.row_data(2).unwrap().title, "Alpha Song"); // Delta Artist
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_filter_and_sort_songs_title_ascending_uses_original_model() {
    let tmp = TempDir::new("test_title_asc").unwrap();
    let songs = create_test_song_models();

    let result = filter_and_sort_songs(
        songs,
        "",
        SongSortCriterion::Title,
        true,
        100,
        100,
        tmp.path().to_path_buf(),
    );

    assert_eq!(result.row_count(), 3);
    assert_eq!(result.row_data(0).unwrap().title, "Bravo Song");
    assert_eq!(result.row_data(1).unwrap().title, "Alpha Song");
    assert_eq!(result.row_data(2).unwrap().title, "Charlie Song");
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_save_queue_creates_playlist_in_db() {
    let tmp = TempDir::new("utils_save_queue_test").unwrap();
    let test_dir = tmp.path().to_path_buf();
    let context = PluginContext {
        data_dir: test_dir.clone(),
        cache_dir: test_dir.clone(),
        tmp_dir: test_dir.clone(),
        #[cfg(target_os = "android")]
        android_context: types::android::AndroidJNIContext::default(),
    };
    let state_manager = StateManager::new_with_context(context).unwrap();

    let song = Song {
        song: Some(InnerSong {
            id: Some("song_util_1".into()),
            title: Some("Queue Song".into()),
            path: Some("/music/test_u1.mp3".into()),
            ..Default::default()
        }),
        ..Default::default()
    };
    {
        let mut ph = state_manager.get_player_handler_mut().await;
        ph.add_to_queue(vec![song]);
    }

    save_queue(
        &state_manager,
        "Custom Playlist".to_string(),
        "Custom Desc".to_string(),
    )
    .await;

    let db = state_manager.get_database().await;
    let playlists_res = db.get_entity_by_options(GetEntityOptions {
        playlist: Some(Playlist::default()),
        ..Default::default()
    });

    assert!(playlists_res.is_ok());
    let res = playlists_res.unwrap().result;
    match res {
        Some(entity_result::Result::Playlists(list)) => {
            assert_eq!(list.playlists.len(), 1);
            assert_eq!(list.playlists[0].playlist_name, "Custom Playlist");
            assert_eq!(list.playlists[0].playlist_desc, Some("Custom Desc".into()));
        }
        _ => panic!("Expected playlists in entity result"),
    }
}
