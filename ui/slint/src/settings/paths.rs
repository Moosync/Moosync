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
use slint::{ComponentHandle, ModelRc, VecModel, invoke_from_event_loop};
use state_manager::StateManager;
use tracing::Instrument;

use crate::{AppPreferences, MainWindow, PreferenceItem, pages::PageHandler};

pub static PREFERENCES: &[&LazyLock<ProtoPreferenceItem>] = &[
    &MUSIC_PATHS,
    &EXCLUDE_MUSIC_PATHS,
    &SCAN_THREADS,
    &ARTIST_SPLITTER,
    &SCAN_INTERVAL,
    &THUMBNAIL_PATH,
    &ARTWORK_PATH,
];

pub struct PathsPageHandler<'a> {
    main_window: &'a MainWindow,
    state_manager: &'a StateManager,
}

impl<'a> PathsPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &'a MainWindow, state_manager: &'a StateManager) -> Self {
        Self {
            main_window,
            state_manager,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_preferences() -> Vec<ProtoPreferenceItem> {
        PREFERENCES.iter().map(|p| (**p).clone()).collect()
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn refresh_preferences(
        main_window_weak: slint::Weak<MainWindow>,
        state_manager: StateManager,
    ) {
        let config = state_manager.get_preference_config().await;
        let items: Vec<ProtoPreferenceItem> =
            PREFERENCES.iter().map(|pref| config.load(pref)).collect();

        if let Err(e) = invoke_from_event_loop(move || {
            if let Some(main_window) = main_window_weak.upgrade() {
                let slint_items: Vec<PreferenceItem> =
                    items.into_iter().map(PreferenceItem::from).collect();
                let prefs_global = main_window.global::<AppPreferences>();
                prefs_global.set_paths_items(ModelRc::new(VecModel::from(slint_items)));
            }
        }) {
            tracing::error!(
                "Failed to invoke from event loop in paths refresh_preferences: {:?}",
                e
            );
        }
    }
}

impl<'a> PageHandler for PathsPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) { self.on_show(); }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) {
        let main_window_weak = self.main_window.as_weak();
        let state_manager = self.state_manager.clone();

        tokio::spawn(
            async move {
                Self::refresh_preferences(main_window_weak, state_manager).await;
            }
            .in_current_span(),
        );
    }
}
