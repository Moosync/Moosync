use slint::{Model, ModelRc, VecModel};
use songs_proto::moosync::types::{Album, Artist, InnerSong, Song};
use tempdir::TempDir;

use super::filter_and_sort_songs;
use crate::{SongModel, SongSortCriterion};

#[tracing::instrument(level = "debug", skip_all)]
fn create_test_song_models() -> ModelRc<SongModel> {
    let song1 = SongModel::from(Song {
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
    });

    let song2 = SongModel::from(Song {
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
    });

    let song3 = SongModel::from(Song {
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
    });

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
