use crate::{Pages, SettingsPages};

pub(crate) trait PageHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) {}
    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) {}
    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppPage {
    AllSongs,
    Albums,
    Artists,
    Playlists,
    Genres,
    Explore,
    Search,
    Paths,
    System,
    Extensions,
    Themes,
    Queue,
    PlaylistContent,
    AlbumContent,
    ArtistContent,
    GenreContent,
}

impl From<Pages> for AppPage {
    fn from(page: Pages) -> Self {
        match page {
            Pages::AllSongs => AppPage::AllSongs,
            Pages::Albums => AppPage::Albums,
            Pages::Artists => AppPage::Artists,
            Pages::Playlists => AppPage::Playlists,
            Pages::Genres => AppPage::Genres,
            Pages::Explore => AppPage::Explore,
            Pages::Search => AppPage::Search,
            Pages::PlaylistContent => AppPage::PlaylistContent,
            Pages::AlbumContent => AppPage::AlbumContent,
            Pages::ArtistContent => AppPage::ArtistContent,
            Pages::GenreContent => AppPage::GenreContent,
        }
    }
}

impl From<SettingsPages> for AppPage {
    fn from(page: SettingsPages) -> Self {
        match page {
            SettingsPages::Paths => AppPage::Paths,
            SettingsPages::System => AppPage::System,
            SettingsPages::Extensions => AppPage::Extensions,
            SettingsPages::Themes => AppPage::Themes,
        }
    }
}
