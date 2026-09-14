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
    AUTO_STARTUP, CLEAR_QUEUE, I18N_LANGUAGE, JUKEBOX_MODE, MINIMIZE_TO_TRAY, VOLUME_PERSIST_MODE,
};
use preferences_proto::moosync::types::PreferenceItem as ProtoPreferenceItem;
use slint::{ComponentHandle, ModelRc, VecModel, invoke_from_event_loop};
use state_manager::StateManager;
use tracing::Instrument;

use crate::{AppPreferences, MainWindow, PreferenceItem, pages::PageHandler};

pub static PREFERENCES: &[&LazyLock<ProtoPreferenceItem>] = &[
    &AUTO_STARTUP,
    &MINIMIZE_TO_TRAY,
    &JUKEBOX_MODE,
    &CLEAR_QUEUE,
    &VOLUME_PERSIST_MODE,
    &I18N_LANGUAGE,
];

pub struct SystemPageHandler<'a> {
    main_window: &'a MainWindow,
    state_manager: &'a StateManager,
}

impl<'a> SystemPageHandler<'a> {
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
                prefs_global.set_system_items(ModelRc::new(VecModel::from(slint_items)));
            }
        }) {
            tracing::error!(
                "Failed to invoke from event loop in system refresh_preferences: {:?}",
                e
            );
        }
    }
}

impl<'a> PageHandler for SystemPageHandler<'a> {
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
