use assertables::assert_some;
use rstest::{fixture, rstest};
use slint::{Image, Model};
use songs_proto::moosync::types::Album;
use tempdir::TempDir;
use tracing_test::traced_test;

use super::lazy_model::{LazyModel, LazySongVecModel, make_lazy_card_model, make_lazy_song_model};
use crate::{
    AlbumModel, ArtistModel, ExtensionItem, GenreModel, MainWindow, PlaylistModel, SongModel,
    test_utils::{TestSlintSmContext, main_window, state_manager_fixture},
};

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn temp_dir_fixture() -> TempDir {
    TempDir::new("moosync_lazy_model_test").expect("failed to create temp dir")
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_lazy_song_vec_model_row_count_and_data(temp_dir_fixture: TempDir) {
    let album: AlbumModel = Album {
        album_id: Some("a1".to_string()),
        album_name: Some("Album 1".to_string()),
        ..Default::default()
    }
    .into();
    let lazy_model = LazySongVecModel::new(
        vec![album],
        100,
        100,
        temp_dir_fixture.path().to_path_buf(),
        false,
    );

    let item = lazy_model.row_data(0);

    assert_eq!(lazy_model.row_count(), 1);
    assert_some!(item.as_ref());
    assert_eq!(item.unwrap().title, "Album 1");
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_lazy_model_set_row_data_and_tracker(temp_dir_fixture: TempDir) {
    let album1: AlbumModel = Album {
        album_id: Some("a1".to_string()),
        album_name: Some("Album 1".to_string()),
        ..Default::default()
    }
    .into();
    let album2: AlbumModel = Album {
        album_id: Some("a2".to_string()),
        album_name: Some("Album 2".to_string()),
        ..Default::default()
    }
    .into();
    let lazy_model = LazySongVecModel::new(
        vec![album1],
        100,
        100,
        temp_dir_fixture.path().to_path_buf(),
        true,
    );

    lazy_model.set_row_data(0, album2);
    lazy_model.set_row_data(99, AlbumModel::default());

    assert_eq!(lazy_model.row_data(0).unwrap().title, "Album 2");
    assert!(lazy_model.as_any().is::<LazySongVecModel<AlbumModel>>());
    let _tracker = lazy_model.model_tracker();
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_lazy_song_and_card_helpers(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let song = SongModel {
        id: "s1".into(),
        title: "Song 1".into(),
        ..Default::default()
    };
    let album = AlbumModel {
        id: "a1".into(),
        title: "Album 1".into(),
        ..Default::default()
    };

    let song_model = make_lazy_song_model(&main_window, &sm, vec![song]);
    let card_model = make_lazy_card_model(&main_window, &sm, vec![album]);

    assert_eq!(song_model.row_count(), 1);
    assert_eq!(card_model.row_count(), 1);
    assert_eq!(song_model.row_data(0).unwrap().title, "Song 1");
    assert_eq!(card_model.row_data(0).unwrap().title, "Album 1");
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_lazy_model_trait_implementations() {
    let mut song = SongModel {
        coverPathUrlLow: "http://cover.jpg".into(),
        ..Default::default()
    };
    assert!(!song.has_image_set());
    assert_eq!(song.get_cover_url().as_str(), "http://cover.jpg");
    song.set_cover(Image::default());
    let _ = song.get_image();

    let mut album = AlbumModel {
        coverPathUrl: "http://album.jpg".into(),
        ..Default::default()
    };
    assert_eq!(album.get_cover_url().as_str(), "http://album.jpg");
    album.set_cover(Image::default());
    let _ = album.get_image();

    let mut artist = ArtistModel {
        coverPathUrl: "http://artist.jpg".into(),
        ..Default::default()
    };
    assert_eq!(artist.get_cover_url().as_str(), "http://artist.jpg");
    artist.set_cover(Image::default());
    let _ = artist.get_image();

    let mut genre = GenreModel {
        coverPathUrl: "http://genre.jpg".into(),
        ..Default::default()
    };
    assert_eq!(genre.get_cover_url().as_str(), "http://genre.jpg");
    genre.set_cover(Image::default());
    let _ = genre.get_image();

    let mut playlist = PlaylistModel {
        coverPathUrl: "http://playlist.jpg".into(),
        ..Default::default()
    };
    assert_eq!(playlist.get_cover_url().as_str(), "http://playlist.jpg");
    playlist.set_cover(Image::default());
    let _ = playlist.get_image();

    let mut ext = ExtensionItem {
        icon_url: "http://ext.png".into(),
        ..Default::default()
    };
    assert_eq!(ext.get_cover_url().as_str(), "http://ext.png");
    ext.set_cover(Image::default());
    let _ = ext.get_image();
}
