use rstest::rstest;
use slint::ComponentHandle;
use tracing_test::traced_test;

use super::navigation::{goto_album, goto_artist};
use crate::{
    AlbumModel, AlbumsPageProps, ArtistModel, ArtistsPageProps, MainWindow, Pages,
    test_utils::main_window,
};

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_goto_album_sets_selected_album_and_active_page(main_window: MainWindow) {
    let album = AlbumModel {
        id: "album_123".into(),
        title: "Test Album".into(),
        songs_count: 5,
        ..Default::default()
    };

    goto_album(&main_window, album.clone());

    assert_eq!(
        main_window
            .global::<AlbumsPageProps>()
            .get_selected_album()
            .id,
        "album_123"
    );
    assert_eq!(main_window.get_active_page(), Pages::AlbumContent);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_goto_artist_sets_selected_artist_and_active_page(main_window: MainWindow) {
    let artist = ArtistModel {
        id: "artist_456".into(),
        title: "Test Artist".into(),
        songs_count: 12,
        ..Default::default()
    };

    goto_artist(&main_window, artist.clone());

    assert_eq!(
        main_window
            .global::<ArtistsPageProps>()
            .get_selected_artist()
            .id,
        "artist_456"
    );
    assert_eq!(main_window.get_active_page(), Pages::ArtistContent);
}
