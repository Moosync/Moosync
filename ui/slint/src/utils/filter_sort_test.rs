use rstest::{fixture, rstest};
use slint::{Model, ModelRc, VecModel};
use songs_proto::moosync::types::{Album, Artist, InnerSong, Song};
use tempdir::TempDir;
use tracing_test::traced_test;

use super::filter_and_sort_songs;
use crate::{SongModel, SongSortCriterion};

struct TestFilterSortContext {
    pub _temp_dir: TempDir,
    pub songs: ModelRc<SongModel>,
}

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn filter_sort_context() -> TestFilterSortContext {
    let temp_dir = TempDir::new("test_filter_sort").expect("failed to create temp dir");

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

    let songs = ModelRc::new(VecModel::from(vec![song1, song2, song3]));

    TestFilterSortContext {
        _temp_dir: temp_dir,
        songs,
    }
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_filter_and_sort_songs_filter_by_title(filter_sort_context: TestFilterSortContext) {
    let TestFilterSortContext { _temp_dir, songs } = filter_sort_context;

    let result = filter_and_sort_songs(
        songs,
        "Bravo",
        SongSortCriterion::Title,
        true,
        100,
        100,
        _temp_dir.path().to_path_buf(),
    );

    assert_eq!(result.row_count(), 1);
    assert_eq!(result.row_data(0).unwrap().title, "Bravo Song");
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_filter_and_sort_songs_search_matches_only_title(
    filter_sort_context: TestFilterSortContext,
) {
    let TestFilterSortContext { _temp_dir, songs } = filter_sort_context;

    let result_title = filter_and_sort_songs(
        songs.clone(),
        "Alpha",
        SongSortCriterion::Title,
        true,
        100,
        100,
        _temp_dir.path().to_path_buf(),
    );
    let result_artist = filter_and_sort_songs(
        songs.clone(),
        "Delta",
        SongSortCriterion::Title,
        true,
        100,
        100,
        _temp_dir.path().to_path_buf(),
    );
    let result_album = filter_and_sort_songs(
        songs,
        "Echo",
        SongSortCriterion::Title,
        true,
        100,
        100,
        _temp_dir.path().to_path_buf(),
    );

    assert_eq!(result_title.row_count(), 1);
    assert_eq!(result_title.row_data(0).unwrap().title, "Alpha Song");
    assert_eq!(result_artist.row_count(), 0);
    assert_eq!(result_album.row_count(), 0);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_filter_and_sort_songs_sort_by_title(filter_sort_context: TestFilterSortContext) {
    let TestFilterSortContext { _temp_dir, songs } = filter_sort_context;

    let result = filter_and_sort_songs(
        songs,
        "Song",
        SongSortCriterion::Title,
        true,
        100,
        100,
        _temp_dir.path().to_path_buf(),
    );

    assert_eq!(result.row_count(), 3);
    assert_eq!(result.row_data(0).unwrap().title, "Alpha Song");
    assert_eq!(result.row_data(1).unwrap().title, "Bravo Song");
    assert_eq!(result.row_data(2).unwrap().title, "Charlie Song");
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_filter_and_sort_songs_sort_by_title_descending(filter_sort_context: TestFilterSortContext) {
    let TestFilterSortContext { _temp_dir, songs } = filter_sort_context;

    let result = filter_and_sort_songs(
        songs,
        "",
        SongSortCriterion::Title,
        false,
        100,
        100,
        _temp_dir.path().to_path_buf(),
    );

    assert_eq!(result.row_count(), 3);
    assert_eq!(result.row_data(0).unwrap().title, "Charlie Song");
    assert_eq!(result.row_data(1).unwrap().title, "Bravo Song");
    assert_eq!(result.row_data(2).unwrap().title, "Alpha Song");
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_filter_and_sort_songs_sort_by_date(filter_sort_context: TestFilterSortContext) {
    let TestFilterSortContext { _temp_dir, songs } = filter_sort_context;

    let result = filter_and_sort_songs(
        songs,
        "",
        SongSortCriterion::Date,
        true,
        100,
        100,
        _temp_dir.path().to_path_buf(),
    );

    assert_eq!(result.row_count(), 3);
    assert_eq!(result.row_data(0).unwrap().title, "Charlie Song");
    assert_eq!(result.row_data(1).unwrap().title, "Bravo Song");
    assert_eq!(result.row_data(2).unwrap().title, "Alpha Song");
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_filter_and_sort_songs_sort_by_album(filter_sort_context: TestFilterSortContext) {
    let TestFilterSortContext { _temp_dir, songs } = filter_sort_context;

    let result = filter_and_sort_songs(
        songs,
        "",
        SongSortCriterion::Album,
        true,
        100,
        100,
        _temp_dir.path().to_path_buf(),
    );

    assert_eq!(result.row_count(), 3);
    assert_eq!(result.row_data(0).unwrap().album_name, "Alpha Album");
    assert_eq!(result.row_data(1).unwrap().album_name, "Echo Album");
    assert_eq!(result.row_data(2).unwrap().album_name, "Zulu Album");
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_filter_and_sort_songs_sort_by_track_number(filter_sort_context: TestFilterSortContext) {
    let TestFilterSortContext { _temp_dir, songs } = filter_sort_context;

    let result = filter_and_sort_songs(
        songs,
        "",
        SongSortCriterion::TrackNumber,
        true,
        100,
        100,
        _temp_dir.path().to_path_buf(),
    );

    assert_eq!(result.row_count(), 3);
    assert_eq!(result.row_data(0).unwrap().track_no, 1.0);
    assert_eq!(result.row_data(1).unwrap().track_no, 2.0);
    assert_eq!(result.row_data(2).unwrap().track_no, 3.0);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_filter_and_sort_songs_sort_by_artist(filter_sort_context: TestFilterSortContext) {
    let TestFilterSortContext { _temp_dir, songs } = filter_sort_context;

    let result = filter_and_sort_songs(
        songs,
        "",
        SongSortCriterion::Artist,
        true,
        100,
        100,
        _temp_dir.path().to_path_buf(),
    );

    assert_eq!(result.row_count(), 3);
    assert_eq!(result.row_data(0).unwrap().title, "Charlie Song");
    assert_eq!(result.row_data(1).unwrap().title, "Bravo Song");
    assert_eq!(result.row_data(2).unwrap().title, "Alpha Song");
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_filter_and_sort_songs_title_ascending_uses_original_model(
    filter_sort_context: TestFilterSortContext,
) {
    let TestFilterSortContext { _temp_dir, songs } = filter_sort_context;

    let result = filter_and_sort_songs(
        songs,
        "",
        SongSortCriterion::Title,
        true,
        100,
        100,
        _temp_dir.path().to_path_buf(),
    );

    assert_eq!(result.row_count(), 3);
    assert_eq!(result.row_data(0).unwrap().title, "Bravo Song");
    assert_eq!(result.row_data(1).unwrap().title, "Alpha Song");
    assert_eq!(result.row_data(2).unwrap().title, "Charlie Song");
}
