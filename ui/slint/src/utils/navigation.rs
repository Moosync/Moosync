//! Navigation utilities for entity navigation.
//!
//! NOTE: Do not call `MainWindow::set_active_page` manually elsewhere in the
//! codebase. All page transitions must route through `NavigationManager` /
//! `NavigationHistory` to ensure history, UI buttons, and lifecycle events are
//! properly synchronized.

use slint::ComponentHandle;

use crate::{
    AlbumModel, AlbumsPageProps, AppProps, ArtistModel, ArtistsPageProps, MainWindow, Pages,
};

#[tracing::instrument(level = "debug", skip_all)]
pub fn goto_album(main_window: &MainWindow, album: AlbumModel) {
    tracing::debug!("Setting goto album: {:?}", album);
    main_window
        .global::<AlbumsPageProps>()
        .set_selected_album(album);
    main_window
        .global::<AppProps>()
        .set_active_page(Pages::AlbumContent);
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn goto_artist(main_window: &MainWindow, artist: ArtistModel) {
    tracing::debug!("Setting goto artist: {:?}", artist);
    main_window
        .global::<ArtistsPageProps>()
        .set_selected_artist(artist);
    main_window
        .global::<AppProps>()
        .set_active_page(Pages::ArtistContent);
}
