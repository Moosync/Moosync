use crate::MainWindow;
use crate::pages::PageHandler;
use slint::{ComponentHandle, Model, ModelRc};
use state_manager::StateManager;

pub struct ExtensionsPageHandler<'a> {
    main_window: &'a MainWindow,
    state_manager: &'a StateManager,
}

impl<'a> ExtensionsPageHandler<'a> {
    pub fn new(main_window: &'a MainWindow, state_manager: &'a StateManager) -> Self {
        Self {
            main_window,
            state_manager,
        }
    }

    fn setup_callbacks(&self) {
        let sm = self.state_manager.clone();
        let mw = self.main_window.as_weak();

        let sm_cl = sm.clone();
        let mw_cl = mw.clone();
        self.main_window
            .global::<crate::AppCallbacks>()
            .on_toggle_extension(move |package_name| {
                let package_name = package_name.to_string();
                let main_window_weak = mw_cl.clone();
                let state_manager = sm_cl.clone();
                tokio::spawn(async move {
                    handle_toggle_extension(package_name, main_window_weak, state_manager).await;
                });
            });

        self.main_window
            .global::<crate::AppCallbacks>()
            .on_install_extension(move |file_path| {
                let file_path = file_path.to_string();
                let main_window_weak = mw.clone();
                let state_manager = sm.clone();
                tokio::spawn(async move {
                    install_local_extension(file_path, main_window_weak, state_manager).await;
                });
            });
    }
}

impl<'a> PageHandler for ExtensionsPageHandler<'a> {
    fn initialize(&self) {
        tracing::info!("ExtensionsPageHandler: Initializing settings page handler");
        self.setup_callbacks();

        let main_window_weak = self.main_window.as_weak();
        let state_manager_clone = self.state_manager.clone();
        self.state_manager.on_extensions_updated(move || {
            let main_window_weak = main_window_weak.clone();
            let state_manager = state_manager_clone.clone();
            refresh_extensions_list(main_window_weak, state_manager);
        });

        refresh_extensions_list(self.main_window.as_weak(), self.state_manager.clone());
    }

    fn on_show(&self) {
        tracing::info!("ExtensionsPageHandler: on_show triggered");
        refresh_extensions_list(self.main_window.as_weak(), self.state_manager.clone());
    }

    fn on_hide(&self) {
        tracing::info!("ExtensionsPageHandler: on_hide triggered");
    }
}

async fn install_local_extension(
    file_path: String,
    main_window_weak: slint::Weak<crate::MainWindow>,
    state_manager: StateManager,
) {
    tracing::info!("install_local_extension: Installing from {}", file_path);
    let handler = state_manager.get_extension_handler().await;
    match handler.inner.install_extension(file_path) {
        Ok(_) => {
            tracing::info!("install_local_extension: Successfully installed");
            refresh_extensions_list(main_window_weak, state_manager);
        }
        Err(e) => {
            tracing::error!("install_local_extension: Failed to install: {:?}", e);
        }
    }
}

async fn get_extension_ui_details(
    package_name: &str,
    main_window_weak: &slint::Weak<crate::MainWindow>,
) -> Option<(bool, bool)> {
    let (tx, rx) = tokio::sync::oneshot::channel::<(bool, bool)>();
    let package_name = package_name.to_string();
    let main_window_weak = main_window_weak.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(main_window) = main_window_weak.upgrade() {
            let model = main_window.get_extensions();
            for item in model.iter() {
                if item.package_name.as_str() == package_name {
                    let _ = tx.send((item.is_installed, item.active));
                    return;
                }
            }
        }
    });
    rx.await.ok()
}

fn set_extension_loading_in_ui(
    package_name: String,
    main_window_weak: slint::Weak<crate::MainWindow>,
) {
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(main_window) = main_window_weak.upgrade() {
            let model = main_window.get_extensions();
            for (row, item) in model.iter().enumerate() {
                if item.package_name.as_str() == package_name {
                    let mut new_item = item.clone();
                    new_item.loading = true;
                    model.set_row_data(row, new_item);
                    break;
                }
            }
        }
    });
}

async fn download_and_install_remote(
    package_name: String,
    main_window_weak: slint::Weak<crate::MainWindow>,
    state_manager: StateManager,
) {
    tracing::info!("download_and_install_remote: Downloading {}", package_name);
    set_extension_loading_in_ui(package_name.clone(), main_window_weak.clone());

    let handler = state_manager.get_extension_handler().await;
    let manifest = handler
        .inner
        .get_cached_remote_manifests()
        .into_iter()
        .find(|m| m.package_name == package_name);

    if let Some(manifest) = manifest {
        match handler.inner.download_extension(manifest).await {
            Ok(_) => {
                let _ = handler
                    .inner
                    .set_extension_active(package_name.clone(), true);
            }
            Err(e) => {
                tracing::error!("Failed to download and install extension: {:?}", e);
            }
        }
    } else {
        tracing::error!("No manifest found in cache for {}", package_name);
    }
    refresh_extensions_list(main_window_weak, state_manager);
}

async fn toggle_installed_active(
    package_name: String,
    active: bool,
    main_window_weak: slint::Weak<crate::MainWindow>,
    state_manager: StateManager,
) {
    let new_active = !active;
    tracing::info!(
        "Setting active = {} for extension {}",
        new_active,
        package_name
    );
    let handler = state_manager.get_extension_handler().await;
    match handler
        .inner
        .set_extension_active(package_name.clone(), new_active)
    {
        Ok(_) => {
            refresh_extensions_list(main_window_weak, state_manager);
        }
        Err(e) => {
            tracing::error!("Failed to toggle extension active state: {:?}", e);
        }
    }
}

async fn handle_toggle_extension(
    package_name: String,
    main_window_weak: slint::Weak<crate::MainWindow>,
    state_manager: StateManager,
) {
    tracing::info!(
        "handle_toggle_extension: Requested toggle for {}",
        package_name
    );
    let (is_installed, active) =
        match get_extension_ui_details(&package_name, &main_window_weak).await {
            Some(res) => res,
            None => {
                tracing::error!("Failed to get extension details from event loop");
                return;
            }
        };

    if is_installed {
        toggle_installed_active(package_name, active, main_window_weak, state_manager).await;
    } else {
        download_and_install_remote(package_name, main_window_weak, state_manager).await;
    }
}

fn sort_extension_items(vector: &mut Vec<crate::ExtensionItem>) {
    vector.sort_by(|a, b| {
        let rank = |item: &crate::ExtensionItem| {
            if item.is_installed && item.active && !item.has_started {
                0 // Installing / Spawning
            } else if item.active {
                1 // Active & started
            } else {
                2 // Inactive / disabled / remote
            }
        };
        rank(a).cmp(&rank(b)).then_with(|| a.name.cmp(&b.name))
    });
}

fn render_extensions_ui(
    main_window_weak: slint::Weak<crate::MainWindow>,
    installed: Vec<extensions_proto::moosync::types::ExtensionDetail>,
    remote: Vec<extensions_proto::moosync::types::FetchedExtensionManifest>,
    cache_dir: std::path::PathBuf,
) {
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(main_window) = main_window_weak.upgrade() {
            let mut extensions_vector = Vec::new();
            for ext in &installed {
                extensions_vector.push(crate::utils::to_extension_item(ext));
            }
            for ext in &remote {
                if !installed.iter().any(|i| i.package_name == ext.package_name) {
                    extensions_vector.push(crate::utils::to_fetched_extension_item(ext));
                }
            }
            sort_extension_items(&mut extensions_vector);
            let model = ModelRc::new(crate::utils::LazySongVecModel::new(
                extensions_vector,
                80,
                0,
                cache_dir,
            ));
            main_window.set_extensions(model);
        }
    });
}

async fn load_and_render_initial_list(
    main_window_weak: slint::Weak<crate::MainWindow>,
    state_manager: StateManager,
) {
    let handler = state_manager.get_extension_handler().await;
    let installed = handler
        .inner
        .get_installed_extensions()
        .unwrap_or_else(|e| {
            tracing::error!("Failed to get installed extensions: {:?}", e);
            vec![]
        });
    let cached_remote = handler.inner.get_cached_remote_manifests();
    let cache_dir = state_manager.get_cache_dir();

    render_extensions_ui(main_window_weak, installed, cached_remote, cache_dir);
}

async fn fetch_and_render_network_list(
    main_window_weak: slint::Weak<crate::MainWindow>,
    state_manager: StateManager,
) {
    let handler = state_manager.get_extension_handler().await;
    let remote = match handler.inner.get_extension_manifest().await {
        Ok(exts) => exts,
        Err(e) => {
            tracing::error!("Failed to fetch remote extensions from network: {:?}", e);
            return;
        }
    };
    let installed = handler
        .inner
        .get_installed_extensions()
        .unwrap_or_else(|e| {
            tracing::error!("Failed to get installed extensions post-fetch: {:?}", e);
            vec![]
        });
    let cache_dir = state_manager.get_cache_dir();
    render_extensions_ui(main_window_weak, installed, remote, cache_dir);
}

fn refresh_extensions_list(
    main_window_weak: slint::Weak<crate::MainWindow>,
    state_manager: StateManager,
) {
    tracing::info!("refresh_extensions_list: Starting reload");
    tokio::spawn(async move {
        load_and_render_initial_list(main_window_weak.clone(), state_manager.clone()).await;
        fetch_and_render_network_list(main_window_weak, state_manager).await;
    });
}
