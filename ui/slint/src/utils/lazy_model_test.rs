use slint::Model;
use songs_proto::moosync::types::Album;
use tempdir::TempDir;

use super::lazy_model::LazySongVecModel;
use crate::AlbumModel;

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_lazy_song_vec_model_row_count_and_data() {
    let tmp = TempDir::new("moosync_lazy_model_test").unwrap();
    let album: AlbumModel = Album {
        album_id: Some("a1".to_string()),
        album_name: Some("Album 1".to_string()),
        ..Default::default()
    }
    .into();
    let lazy_model = LazySongVecModel::new(vec![album], 100, 100, tmp.path().to_path_buf());

    assert_eq!(lazy_model.row_count(), 1);
    let item = lazy_model.row_data(0);
    assert!(item.is_some());
    assert_eq!(item.unwrap().title, "Album 1");
}
