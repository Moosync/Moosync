use extensions::ExtensionHandler;
use extensions_proto::moosync::types::{ExtensionDetail, ExtensionProviderScope};
use slint::{ComponentHandle, ModelRc};
use songs_proto::moosync::types::Song;
use state_manager::StateManager;

use crate::{ExtensionProviderItem, MainWindow, SongModel, Theme, utils::LazySongVecModel};

#[tracing::instrument(level = "debug", skip_all)]
pub fn make_lazy_song_model(
    main_window: &MainWindow,
    state_manager: &StateManager,
    songs: Vec<SongModel>,
) -> ModelRc<SongModel> {
    let theme = main_window.global::<Theme>();
    let cache_dir = state_manager.get_cache_dir();
    ModelRc::new(LazySongVecModel::new(
        songs,
        theme.get_songListItemHeight() as usize,
        theme.get_songListItemWidth() as usize,
        cache_dir,
    ))
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn map_songs_to_models(songs: Vec<Song>, detail: Option<&ExtensionDetail>) -> Vec<SongModel> {
    songs.into_iter().map(|s| (s, detail).into()).collect()
}

#[tracing::instrument(level = "debug", skip_all)]
pub async fn fetch_scope_providers(
    ext_handler: &ExtensionHandler,
    scope: ExtensionProviderScope,
    current_extension: &str,
) -> (Vec<ExtensionProviderItem>, Option<ExtensionDetail>) {
    let active_exts = ext_handler.get_extensions_with_scope(scope).await;

    let providers: Vec<ExtensionProviderItem> = active_exts
        .into_iter()
        .map(|ext| {
            let detail = ext.get_extension_detail();
            let enabled = !current_extension.is_empty() && current_extension == detail.package_name;
            ExtensionProviderItem {
                id: detail.package_name.into(),
                name: detail.name.into(),
                enabled,
            }
        })
        .collect();

    let detail = if !current_extension.is_empty() {
        ext_handler
            .get_extension(current_extension)
            .ok()
            .map(|e| e.get_extension_detail())
    } else {
        None
    };

    (providers, detail)
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn update_provider_list_enabled(
    providers: &[ExtensionProviderItem],
    package_name: &str,
    enabled: bool,
) -> Vec<ExtensionProviderItem> {
    providers
        .iter()
        .cloned()
        .map(|mut p| {
            if p.id == package_name {
                p.enabled = enabled;
            }
            p
        })
        .collect()
}
