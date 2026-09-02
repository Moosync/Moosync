use assertables::assert_some;
use rstest::{fixture, rstest};
use slint::Model;
use songs_proto::moosync::types::Album;
use tempdir::TempDir;
use tracing_test::traced_test;

use super::lazy_model::LazySongVecModel;
use crate::AlbumModel;

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
    let lazy_model =
        LazySongVecModel::new(vec![album], 100, 100, temp_dir_fixture.path().to_path_buf());

    let item = lazy_model.row_data(0);

    assert_eq!(lazy_model.row_count(), 1);
    assert_some!(item.as_ref());
    assert_eq!(item.unwrap().title, "Album 1");
}
