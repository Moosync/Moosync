// Moosync
// Copyright (C) 2024, 2025  Moosync <support@moosync.app>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use crate::{Pages, QueuePages, SettingsPages, pages::AppPage};

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_app_page_from_conversions() {
    assert_eq!(AppPage::from(Pages::AllSongs), AppPage::AllSongs);
    assert_eq!(AppPage::from(Pages::Albums), AppPage::Albums);
    assert_eq!(AppPage::from(Pages::Artists), AppPage::Artists);
    assert_eq!(AppPage::from(Pages::Playlists), AppPage::Playlists);
    assert_eq!(AppPage::from(Pages::Genres), AppPage::Genres);
    assert_eq!(AppPage::from(Pages::Explore), AppPage::Explore);
    assert_eq!(AppPage::from(Pages::Search), AppPage::Search);
    assert_eq!(
        AppPage::from(Pages::PlaylistContent),
        AppPage::PlaylistContent
    );
    assert_eq!(AppPage::from(Pages::AlbumContent), AppPage::AlbumContent);
    assert_eq!(AppPage::from(Pages::ArtistContent), AppPage::ArtistContent);
    assert_eq!(AppPage::from(Pages::GenreContent), AppPage::GenreContent);

    assert_eq!(AppPage::from(SettingsPages::Paths), AppPage::Paths);
    assert_eq!(AppPage::from(SettingsPages::System), AppPage::System);
    assert_eq!(
        AppPage::from(SettingsPages::Extensions),
        AppPage::Extensions
    );
    assert_eq!(AppPage::from(SettingsPages::Themes), AppPage::Themes);

    assert_eq!(AppPage::from(QueuePages::Queue), AppPage::Queue);
}
