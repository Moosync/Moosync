use slint::ComponentHandle;

use crate::{
    AlbumModel, AlbumsPageProps, ArtistModel, ArtistsPageProps, MainWindow, Pages, PlaylistModel,
    PlaylistsPageProps,
};

#[tracing::instrument(level = "debug", skip_all)]
pub fn goto_album(main_window: &MainWindow, album: AlbumModel) {
    main_window
        .global::<AlbumsPageProps>()
        .set_selected_album(album);
    main_window.set_active_page(Pages::AlbumContent);
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn goto_artist(main_window: &MainWindow, artist: ArtistModel) {
    main_window
        .global::<ArtistsPageProps>()
        .set_selected_artist(artist);
    main_window.set_active_page(Pages::ArtistContent);
}

#[allow(dead_code)]
#[tracing::instrument(level = "debug", skip_all)]
pub fn goto_playlist(main_window: &MainWindow, playlist: PlaylistModel) {
    main_window
        .global::<PlaylistsPageProps>()
        .set_selected_playlist(playlist);
    main_window.set_active_page(Pages::PlaylistContent);
}
