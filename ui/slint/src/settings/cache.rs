use std::rc::Rc;

use cache::{
    CacheCategory, CacheManager, CacheReport, InstalledExtensionInfo,
    OrphanCacheKind as CoreOrphanKind,
};
use humansize::{BINARY, format_size};
use slint::{ComponentHandle, ModelRc, VecModel};
use state_manager::StateManager;
use tracing::Instrument;

use crate::{
    CacheItem, CacheState, ExtensionCacheItem, MainWindow, OrphanCacheItem, pages::PageHandler,
    utils::models::OrphanCacheTarget,
};

#[tracing::instrument(level = "debug", skip_all)]
fn get_installed_extensions(state_manager: &StateManager) -> Vec<InstalledExtensionInfo> {
    state_manager
        .get_extension_handler()
        .get_installed_extensions()
        .into_iter()
        .map(|ext| InstalledExtensionInfo {
            package_name: ext.package_name,
            display_name: ext.name,
        })
        .collect()
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn report_to_ui_models(
    report: &CacheReport,
) -> (
    String,
    Vec<CacheItem>,
    Vec<ExtensionCacheItem>,
    Vec<OrphanCacheItem>,
) {
    (
        format_size(report.total_bytes, BINARY),
        report.app_caches.iter().map(Into::into).collect(),
        report.extension_caches.iter().map(Into::into).collect(),
        report.orphan_caches.iter().map(Into::into).collect(),
    )
}

#[tracing::instrument(level = "debug", skip_all)]
fn apply_report_to_ui(main_window_weak: &slint::Weak<MainWindow>, report: CacheReport) {
    let (total_text, app_items, ext_items, orphan_items) = report_to_ui_models(&report);
    let _ = main_window_weak.upgrade_in_event_loop(move |main_window| {
        let state = main_window.global::<CacheState>();
        state.set_total_size_text(total_text.into());
        state.set_category_caches(ModelRc::from(Rc::new(VecModel::from(app_items))));
        state.set_extension_caches(ModelRc::from(Rc::new(VecModel::from(ext_items))));
        state.set_orphan_caches(ModelRc::from(Rc::new(VecModel::from(orphan_items))));
        state.set_is_loading(false);
    });
}

#[tracing::instrument(level = "debug", skip_all)]
async fn refresh_cache_report(
    main_window_weak: slint::Weak<MainWindow>,
    state_manager: StateManager,
) {
    let _ = main_window_weak.upgrade_in_event_loop(|main_window| {
        main_window.global::<CacheState>().set_is_loading(true);
    });

    let installed = get_installed_extensions(&state_manager);
    let manager = CacheManager::new(state_manager.get_cache_dir());
    match manager.get_cache_report(&installed).await {
        Ok(report) => apply_report_to_ui(&main_window_weak, report),
        Err(err) => {
            tracing::error!("Failed to compute cache report: {:?}", err);
            let _ = main_window_weak.upgrade_in_event_loop(|main_window| {
                main_window.global::<CacheState>().set_is_loading(false);
            });
        }
    }
}

pub struct CachePageHandler {
    main_window: MainWindow,
    state_manager: StateManager,
}

impl CachePageHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &MainWindow, state_manager: &StateManager) -> Self {
        Self::setup_callbacks(main_window, state_manager);
        Self {
            main_window: main_window.clone_strong(),
            state_manager: state_manager.clone(),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn setup_callbacks(main_window: &MainWindow, state_manager: &StateManager) {
        let cache_state = main_window.global::<CacheState>();

        let weak = main_window.as_weak();
        let sm_refresh = state_manager.clone();
        cache_state.on_refresh_cache(move || {
            let weak = weak.clone();
            let sm_clone = sm_refresh.clone();
            tokio::spawn(
                async move {
                    refresh_cache_report(weak, sm_clone).await;
                }
                .in_current_span(),
            );
        });

        let weak = main_window.as_weak();
        let sm_app = state_manager.clone();
        cache_state.on_clear_cache_category(move |category| {
            let weak = weak.clone();
            let sm_clone = sm_app.clone();
            let category: CacheCategory = category.into();
            tokio::spawn(
                async move {
                    let manager = CacheManager::new(sm_clone.get_cache_dir());
                    if let Err(err) = manager.clear_app_cache(category).await {
                        tracing::error!("Failed to clear app cache {:?}: {:?}", category, err);
                    }
                    refresh_cache_report(weak, sm_clone).await;
                }
                .in_current_span(),
            );
        });

        let weak = main_window.as_weak();
        let sm_ext = state_manager.clone();
        cache_state.on_clear_extension_cache(move |package_name| {
            let weak = weak.clone();
            let sm_clone = sm_ext.clone();
            let pkg = package_name.to_string();
            tokio::spawn(
                async move {
                    let manager = CacheManager::new(sm_clone.get_cache_dir());
                    if let Err(err) = manager.clear_extension_cache(&pkg).await {
                        tracing::error!("Failed to clear extension cache {}: {:?}", pkg, err);
                    }
                    refresh_cache_report(weak, sm_clone).await;
                }
                .in_current_span(),
            );
        });

        let weak = main_window.as_weak();
        let sm_orphan = state_manager.clone();
        cache_state.on_clear_orphan_cache(move |kind, package_name| {
            let weak = weak.clone();
            let sm_clone = sm_orphan.clone();
            let core_kind: CoreOrphanKind =
                OrphanCacheTarget(kind, package_name.to_string()).into();
            tokio::spawn(
                async move {
                    let installed = get_installed_extensions(&sm_clone);
                    let manager = CacheManager::new(sm_clone.get_cache_dir());
                    if let Err(err) = manager.clear_orphan_cache(&core_kind, &installed).await {
                        tracing::error!("Failed to clear orphan cache {:?}: {:?}", core_kind, err);
                    }
                    refresh_cache_report(weak, sm_clone).await;
                }
                .in_current_span(),
            );
        });

        let weak = main_window.as_weak();
        let sm_orphans = state_manager.clone();
        cache_state.on_clear_all_orphans(move || {
            let weak = weak.clone();
            let sm_clone = sm_orphans.clone();
            tokio::spawn(
                async move {
                    let installed = get_installed_extensions(&sm_clone);
                    let manager = CacheManager::new(sm_clone.get_cache_dir());
                    if let Err(err) = manager.clear_all_orphan_caches(&installed).await {
                        tracing::error!("Failed to clear all orphan caches: {:?}", err);
                    }
                    refresh_cache_report(weak, sm_clone).await;
                }
                .in_current_span(),
            );
        });

        let weak = main_window.as_weak();
        let sm_all = state_manager.clone();
        cache_state.on_clear_all_cache(move || {
            let weak = weak.clone();
            let sm_clone = sm_all.clone();
            tokio::spawn(
                async move {
                    let manager = CacheManager::new(sm_clone.get_cache_dir());
                    if let Err(err) = manager.clear_all_caches().await {
                        tracing::error!("Failed to clear all caches: {:?}", err);
                    }
                    refresh_cache_report(weak, sm_clone).await;
                }
                .in_current_span(),
            );
        });
    }
}

impl PageHandler for CachePageHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) {}

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) {
        let weak = self.main_window.as_weak();
        let state_manager = self.state_manager.clone();
        tokio::spawn(
            async move {
                refresh_cache_report(weak, state_manager).await;
            }
            .in_current_span(),
        );
    }
}
