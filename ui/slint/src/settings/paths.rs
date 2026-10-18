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

use std::sync::LazyLock;

use preferences::keys::{
    ARTIST_SPLITTER, ARTWORK_PATH, EXCLUDE_MUSIC_PATHS, MUSIC_PATHS, SCAN_INTERVAL, SCAN_THREADS,
    THUMBNAIL_PATH,
};
use preferences_proto::moosync::types::PreferenceItem as ProtoPreferenceItem;
use slint::ComponentHandle;
use state_manager::StateManager;
use tracing::Instrument;

use super::utils::{clone_lazy_preferences, to_preference_model, update_preferences_in_ui};
use crate::{AppPreferences, MainWindow, pages::PageHandler};

pub static PREFERENCES: &[&LazyLock<ProtoPreferenceItem>] = &[
    &MUSIC_PATHS,
    &EXCLUDE_MUSIC_PATHS,
    &SCAN_THREADS,
    &ARTIST_SPLITTER,
    &SCAN_INTERVAL,
    &THUMBNAIL_PATH,
    &ARTWORK_PATH,
];

pub struct PathsPageHandler {
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
}

impl PathsPageHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &'static MainWindow, state_manager: &'static StateManager) -> Self {
        Self {
            main_window,
            state_manager,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_preferences() -> Vec<ProtoPreferenceItem> { clone_lazy_preferences(PREFERENCES) }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn refresh_preferences(
        main_window_weak: slint::Weak<MainWindow>,
        state_manager: &'static StateManager,
    ) {
        update_preferences_in_ui(
            main_window_weak,
            state_manager,
            PREFERENCES,
            "paths",
            |main_window, items| {
                main_window
                    .global::<AppPreferences>()
                    .set_paths_items(to_preference_model(items));
            },
        )
        .await;
    }
}

impl PageHandler for PathsPageHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) { self.on_show(); }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) {
        let main_window_weak = self.main_window.as_weak();
        let state_manager = self.state_manager;

        tokio::spawn(
            async move {
                Self::refresh_preferences(main_window_weak, state_manager).await;
            }
            .in_current_span(),
        );
    }
}
