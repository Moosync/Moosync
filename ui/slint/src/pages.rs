use crate::{Pages, QueuePages, SettingsPages};

pub(crate) trait PageHandler {
    fn initialize(&self);
    fn on_show(&self);
    fn on_hide(&self);
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

impl From<QueuePages> for AppPage {
    fn from(page: QueuePages) -> Self {
        match page {
            QueuePages::Queue => AppPage::Queue,
        }
    }
}
