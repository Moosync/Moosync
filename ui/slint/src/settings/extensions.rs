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

use std::path::PathBuf;

use extensions::ExtensionInfo;
use slint::{ComponentHandle, ModelRc};
use state_manager::StateManager;

use crate::{
    AppCallbacks, ExtensionItem, MainWindow, Theme, pages::PageHandler,
    settings::PreferenceHandler, utils::LazySongVecModel,
};

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
                if item.is_installed && item.active && !item.has_started {
                    0 // Installing / spawning
                } else if item.active {
                    1 // Active and started
                } else {
                    2 // Inactive / disabled / remote
                }
            };
            rank(a).cmp(&rank(b)).then_with(|| a.name.cmp(&b.name))
        });

        let theme = main_window.global::<Theme>();
        main_window.set_extensions(ModelRc::new(LazySongVecModel::new(
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
                    tokio::spawn(async move {
                        Self::handle_toggle_extension(package_name, state_manager).await;
                    });
                }
            });

        self.main_window
            .global::<AppCallbacks>()
            .on_install_extension({
                let state_manager = self.state_manager.clone();
                move |file_path| {
                    let file_path = file_path.to_string();
                    let state_manager = state_manager.clone();
                    tokio::spawn(async move {
                        Self::install_local_extension(file_path, state_manager).await;
                    });
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
            ExtensionInfo::Local(_) => {
                if let Err(e) = handler.toggle_extension(info) {
                    tracing::error!("handle_toggle_extension: Failed to toggle: {:?}", e);
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
}

pref_macro::generate_preferences!(
    "src/settings/extensions_prefs.yaml",
    extensions_items,
    ExtensionsPageHandler
);

impl<'a> PageHandler for ExtensionsPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) {
        tracing::info!("ExtensionsPageHandler: Initializing");
        self.init_preferences();
        self.setup_callbacks();

        let state_manager = self.state_manager.clone();
        let main_window_weak = self.main_window.as_weak();
        tokio::spawn(async move {
            let handler = state_manager.get_extension_handler().await;
            let _cancel = handler.on_extensions_updated({
                let state_manager = state_manager.clone();
                let main_window_weak = main_window_weak.clone();
                move |_| {
                    let state_manager = state_manager.clone();
                    let main_window_weak = main_window_weak.clone();
                    tokio::spawn(async move {
                        let handler = state_manager.get_extension_handler().await;
                        if let Err(e) = handler.get_extension_manifest().await {
                            tracing::error!(
                                "on_extensions_updated: failed to refresh remote manifests: {:?}",
                                e
                            );
                        }
                        let extensions = handler.get_all_extensions();
                        let cache_dir = state_manager.get_cache_dir();
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(main_window) = main_window_weak.upgrade() {
                                Self::render_extensions(&main_window, extensions, cache_dir);
                            }
                        });
                    });
                }
            });
        });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) {
        tracing::info!("ExtensionsPageHandler: on_show");
        let state_manager = self.state_manager.clone();
        let main_window_weak = self.main_window.as_weak();
        tokio::spawn(async move {
            let cache_dir = state_manager.get_cache_dir();
            let handler = state_manager.get_extension_handler().await;
            let extensions = handler.get_all_extensions();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(main_window) = main_window_weak.upgrade() {
                    Self::render_extensions(&main_window, extensions, cache_dir);
                }
            });
        });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) {
        tracing::info!("ExtensionsPageHandler: on_hide");
        self.main_window.set_extensions(ModelRc::default());
    }
}
