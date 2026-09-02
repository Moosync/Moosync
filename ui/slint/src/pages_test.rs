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

use rstest::rstest;
use tracing_test::traced_test;

use crate::{Pages, QueuePages, SettingsPages, pages::AppPage};

#[rstest]
#[case(Pages::AllSongs, AppPage::AllSongs)]
#[case(Pages::Albums, AppPage::Albums)]
#[case(Pages::Artists, AppPage::Artists)]
#[case(Pages::Playlists, AppPage::Playlists)]
#[case(Pages::Genres, AppPage::Genres)]
#[case(Pages::Explore, AppPage::Explore)]
#[case(Pages::Search, AppPage::Search)]
#[case(Pages::PlaylistContent, AppPage::PlaylistContent)]
#[case(Pages::AlbumContent, AppPage::AlbumContent)]
#[case(Pages::ArtistContent, AppPage::ArtistContent)]
#[case(Pages::GenreContent, AppPage::GenreContent)]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_app_page_from_pages(#[case] page: Pages, #[case] expected: AppPage) {
    assert_eq!(AppPage::from(page), expected);
}

#[rstest]
#[case(SettingsPages::Paths, AppPage::Paths)]
#[case(SettingsPages::System, AppPage::System)]
#[case(SettingsPages::Extensions, AppPage::Extensions)]
#[case(SettingsPages::Themes, AppPage::Themes)]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_app_page_from_settings_pages(#[case] page: SettingsPages, #[case] expected: AppPage) {
    assert_eq!(AppPage::from(page), expected);
}

#[rstest]
#[case(QueuePages::Queue, AppPage::Queue)]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_app_page_from_queue_pages(#[case] page: QueuePages, #[case] expected: AppPage) {
    assert_eq!(AppPage::from(page), expected);
}
