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
use slint::Model;
use songs_proto::moosync::types::{
    Album, Artist, Genre, InnerSong, Playlist, SearchResult as ProtoSearchResult, Song,
};
use tempdir::TempDir;

use crate::utils::{
    cache_image, default_empty_icon, default_folder_icon, default_song_cover, get_safe_name,
    parse_color, parse_length, song_model_to_song, to_album_model, to_artist_model,
    to_extension_item, to_fetched_extension_item, to_genre_model, to_playlist_model,
    to_search_result, to_song_model,
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
    let expected = default_song_cover();

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
    let expected = default_song_cover();

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
    let expected = default_song_cover();

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
    let expected = default_song_cover();

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
