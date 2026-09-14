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

use std::{path::PathBuf, sync::LazyLock};

use extensions::ExtensionInfo;
use preferences::keys::EXTENSION_REGISTRIES;
use preferences_proto::moosync::types::PreferenceItem as ProtoPreferenceItem;
use slint::{ComponentHandle, ModelRc, VecModel, invoke_from_event_loop};
use state_manager::StateManager;
use tracing::Instrument;

use crate::{
    AppCallbacks, ExtensionItem, ExtensionPreferenceGroup, ExtensionsPageProps,
    ExtensionsPreferenceProps, MainWindow, PreferenceItem, Theme, pages::PageHandler,
    utils::LazySongVecModel,
};

pub static PREFERENCES: &[&LazyLock<ProtoPreferenceItem>] = &[&EXTENSION_REGISTRIES];

pub struct ExtensionsPageHandler<'a> {
    main_window: &'a MainWindow,
    state_manager: &'a StateManager,
}

impl<'a> ExtensionsPageHandler<'a> {
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
    fn render_extensions(
        main_window: &MainWindow,
        extensions: Vec<ExtensionInfo>,
        cache_dir: PathBuf,
    ) {
        let mut items: Vec<ExtensionItem> = extensions
            .into_iter()
            .map(|ext| match ext {
                ExtensionInfo::Local(detail) => ExtensionItem::from(detail),
                ExtensionInfo::Remote(manifest) => ExtensionItem::from(manifest),
                ExtensionInfo::LocalPath(_) => unreachable!(),
            })
            .collect();

        items.sort_by(|a, b| {
            let rank = |item: &ExtensionItem| {
                if item.is_installed && !item.has_started {
                    0
                } else if item.is_installed {
                    1
                } else {
                    2 // Remote / uninstalled
                }
            };
            rank(a).cmp(&rank(b)).then_with(|| a.name.cmp(&b.name))
        });

        let theme = main_window.global::<Theme>();
        main_window
            .global::<ExtensionsPageProps>()
            .set_extensions(ModelRc::new(LazySongVecModel::new(
                items,
                theme.get_extensionListItemHeight() as usize,
                theme.get_extensionListItemWidth() as usize,
                cache_dir,
            )));
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn setup_callbacks(&self) {
        self.main_window
            .global::<AppCallbacks>()
            .on_toggle_extension({
                let state_manager = self.state_manager.clone();
                move |package_name| {
                    let package_name = package_name.to_string();
                    let state_manager = state_manager.clone();
                    tokio::spawn(
                        async move {
                            Self::handle_toggle_extension(package_name, state_manager).await;
                        }
                        .instrument(tracing::debug_span!("slint_cb_on_toggle_extension")),
                    );
                }
            });

        self.main_window
            .global::<AppCallbacks>()
            .on_install_extension({
                let state_manager = self.state_manager.clone();
                move |file_path| {
                    let file_path = file_path.to_string();
                    let state_manager = state_manager.clone();
                    tokio::spawn(
                        async move {
                            Self::install_local_extension(file_path, state_manager).await;
                        }
                        .instrument(tracing::debug_span!("slint_cb_on_install_extension")),
                    );
                }
            });

        self.main_window
            .global::<AppCallbacks>()
            .on_update_all_extensions({
                let state_manager = self.state_manager.clone();
                let main_window_weak = self.main_window.as_weak();
                move || {
                    let state_manager = state_manager.clone();
                    let main_window_weak = main_window_weak.clone();
                    if let Some(main_window) = main_window_weak.upgrade() {
                        main_window
                            .global::<ExtensionsPageProps>()
                            .set_has_updates(false);
                    }
                    tokio::spawn(
                        async move {
                            tracing::info!("on_update_all_extensions");
                            let mut handler = state_manager.get_extension_handler_mut().await;
                            if let Err(e) = handler.update_all_extensions().await {
                                tracing::error!("on_update_all_extensions: Failed: {:?}", e);
                                handler.check_for_updates();
                                let has_updates = handler.has_updates();
                                let _ = slint::invoke_from_event_loop(move || {
                                    if let Some(main_window) = main_window_weak.upgrade() {
                                        main_window
                                            .global::<ExtensionsPageProps>()
                                            .set_has_updates(has_updates);
                                    }
                                });
                            }
                        }
                        .instrument(tracing::debug_span!("slint_cb_on_update_all_extensions")),
                    );
                }
            });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn handle_toggle_extension(package_name: String, state_manager: StateManager) {
        tracing::info!("handle_toggle_extension: {}", package_name);
        let handler = state_manager.get_extension_handler().await;
        let extensions = handler.get_all_extensions();
        let ext_info = extensions.into_iter().find(|ext| match ext {
            ExtensionInfo::Local(detail) => detail.package_name == package_name,
            ExtensionInfo::Remote(manifest) => manifest.package_name == package_name,
            _ => false,
        });

        let Some(info) = ext_info else {
            return;
        };

        match info {
            ExtensionInfo::Local(detail) => {
                if let Err(e) = handler.remove_extension(detail.package_name) {
                    tracing::error!(
                        "handle_toggle_extension: Failed to remove extension: {:?}",
                        e
                    );
                }
            }
            ExtensionInfo::Remote(_) => {
                if let Err(e) = handler.install_extension(info).await {
                    tracing::error!(
                        "handle_toggle_extension: Failed to install remote extension: {:?}",
                        e
                    );
                }
            }
            _ => {}
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn install_local_extension(file_path: String, state_manager: StateManager) {
        tracing::info!("install_local_extension: {}", file_path);
        let handler = state_manager.get_extension_handler().await;
        let info = ExtensionInfo::LocalPath(PathBuf::from(file_path));
        match handler.install_extension(info).await {
            Ok(_) => tracing::info!("install_local_extension: Installed successfully"),
            Err(e) => tracing::error!("install_local_extension: Failed: {:?}", e),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn refresh_preferences(
        main_window_weak: slint::Weak<MainWindow>,
        state_manager: StateManager,
    ) {
        let config = state_manager.get_preference_config().await;
        let handler = state_manager.get_extension_handler().await;
        let installed = handler.get_installed_extensions();

        let loaded_static: Vec<ProtoPreferenceItem> =
            PREFERENCES.iter().map(|pref| config.load(pref)).collect();

        let mut loaded_ext_groups = Vec::new();
        for ext in installed {
            if ext.preferences.is_empty() {
                continue;
            }

            let display_name = if !ext.name.is_empty() {
                ext.name.clone()
            } else {
                ext.package_name.clone()
            };

            let mut group_items = Vec::new();
            for mut p in ext.preferences {
                p.id = format!("ext:{}:{}", ext.package_name, p.id);
                let loaded = config.load(&p);
                group_items.push(loaded);
            }
            loaded_ext_groups.push((ext.package_name, display_name, group_items));
        }

        let _ = slint::invoke_from_event_loop(move || {
            if let Some(main_window) = main_window_weak.upgrade() {
                let mut extension_groups = Vec::new();
                for (package_name, display_name, group_items) in loaded_ext_groups {
                    let slint_group_items: Vec<PreferenceItem> =
                        group_items.into_iter().map(PreferenceItem::from).collect();
                    extension_groups.push(ExtensionPreferenceGroup {
                        package_name: package_name.into(),
                        display_name: display_name.into(),
                        preferences: ModelRc::new(VecModel::from(slint_group_items)),
                    });
                }

                let slint_static: Vec<PreferenceItem> = loaded_static
                    .into_iter()
                    .map(PreferenceItem::from)
                    .collect();
                let prefs_global = main_window.global::<ExtensionsPreferenceProps>();
                prefs_global.set_static_preferences(ModelRc::new(VecModel::from(slint_static)));
                prefs_global
                    .set_extension_preferences(ModelRc::new(VecModel::from(extension_groups)));
            }
        });
    }
}

impl<'a> PageHandler for ExtensionsPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) {
        tracing::info!("ExtensionsPageHandler: Initializing");
        self.setup_callbacks();

        let state_manager = self.state_manager.clone();
        let main_window_weak = self.main_window.as_weak();
        tokio::spawn(
            async move {
                let handler = state_manager.get_extension_handler().await;
                let _cancel_ext = handler.on_extensions_updated({
                    let state_manager = state_manager.clone();
                    let main_window_weak = main_window_weak.clone();
                    move |_| {
                        let state_manager = state_manager.clone();
                        let main_window_weak = main_window_weak.clone();
                        tokio::spawn(
                            async move {
                                let handler = state_manager.get_extension_handler().await;
                                let extensions = handler.get_all_extensions();
                                let has_updates = handler.has_updates();
                                let cache_dir = state_manager.get_cache_dir();
                                let mw_weak = main_window_weak.clone();
                                if let Err(e) = invoke_from_event_loop(move || {
                                    if let Some(main_window) = mw_weak.upgrade() {
                                        main_window
                                            .global::<ExtensionsPageProps>()
                                            .set_has_updates(has_updates);
                                        Self::render_extensions(
                                            &main_window,
                                            extensions,
                                            cache_dir,
                                        );
                                    }
                                }) {
                                    tracing::error!(
                                        "Failed to invoke from event loop on extensions updated: {:?}",
                                        e
                                    );
                                }
                            }
                            .in_current_span(),
                        );
                    }
                });
            }
            .in_current_span(),
        );
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) {
        tracing::info!("ExtensionsPageHandler: on_show");
        let state_manager = self.state_manager.clone();
        let main_window_weak = self.main_window.as_weak();
        tokio::spawn(
            async move {
                let cache_dir = state_manager.get_cache_dir();
                let handler = state_manager.get_extension_handler().await;
                let extensions = handler.get_all_extensions();
                let has_updates = handler.has_updates();
                let mw_weak = main_window_weak.clone();
                if let Err(e) = invoke_from_event_loop(move || {
                    if let Some(main_window) = mw_weak.upgrade() {
                        main_window
                            .global::<ExtensionsPageProps>()
                            .set_has_updates(has_updates);
                        Self::render_extensions(&main_window, extensions, cache_dir);
                    }
                }) {
                    tracing::error!(
                        "Failed to invoke from event loop on extensions on_show: {:?}",
                        e
                    );
                }
                Self::refresh_preferences(main_window_weak, state_manager).await;
            }
            .in_current_span(),
        );
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) {
        tracing::info!("ExtensionsPageHandler: on_hide");
        self.main_window
            .global::<ExtensionsPageProps>()
            .set_extensions(ModelRc::default());
    }
}
