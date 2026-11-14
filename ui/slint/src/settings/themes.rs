use std::{
    collections::HashMap,
    mem,
    sync::{Arc, Mutex},
    time::Duration,
};

use preferences::keys::{ACTIVE_THEME_ID, PreferenceItemExt, THEME_REGISTRIES};
use preferences_proto::moosync::types::{
    PreferenceItem as ProtoPreferenceItem, PreferenceValue, preference_value,
};
use slint::{ComponentHandle, ModelRc, VecModel};
use state_manager::StateManager;
use themes_proto::moosync::types::{ThemeDetails, ThemeItem, ThemeManifest};
use tracing::Instrument;
use types::prelude::{ThemeExt, ThemeItemExt};

use crate::{
    MainWindow, PreferenceItem, ThemesPageProps,
    pages::PageHandler,
    utils::{parse_color, parse_length, parse_rgba_color},
};

#[cfg(not(target_os = "android"))]
theme_macro::generate_theme_ui_helpers!("ui/slint/src/constants.slint");
#[cfg(target_os = "android")]
theme_macro::generate_theme_ui_helpers!("ui/slint/src/constants_mobile.slint");

#[tracing::instrument(level = "debug", skip_all)]
fn build_theme_config(
    id: String,
    name: String,
    description: String,
    author: String,
    registry: String,
    theme_item: ThemeItem,
) -> crate::ThemeConfig {
    let default_item = types::prelude::get_default_theme_item();
    let get_color = |key: &str| {
        let val = theme_item
            .get_constant(key)
            .or_else(|| default_item.get_constant(key))
            .unwrap_or_default();
        parse_rgba_color(&val)
    };

    crate::ThemeConfig {
        id: id.into(),
        name: name.into(),
        description: description.into(),
        author: author.into(),
        tertiary: get_color("tertiary"),
        accent_color: get_color("accent"),
        primary_color: get_color("primary"),
        secondary_color: get_color("secondary"),
        registry: registry.into(),
    }
}

pub struct ThemesPageHandler {
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
}

impl ThemesPageHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &'static MainWindow, state_manager: &'static StateManager) -> Self {
        Self {
            main_window,
            state_manager,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_preferences() -> Vec<ProtoPreferenceItem> { vec![THEME_REGISTRIES.clone()] }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn refresh_preferences(
        main_window_weak: slint::Weak<MainWindow>,
        state_manager: &'static StateManager,
    ) {
        let config = state_manager.get_preference_config();
        let loaded = config.load(&THEME_REGISTRIES);
        let update_ui = move |main_window: &MainWindow| {
            let slint_pref = PreferenceItem::from(loaded);
            main_window
                .global::<ThemesPageProps>()
                .set_static_preferences(ModelRc::new(VecModel::from(vec![slint_pref])));
        };

        if let Some(main_window) = main_window_weak.upgrade() {
            update_ui(&main_window);
        } else {
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(main_window) = main_window_weak.upgrade() {
                    update_ui(&main_window);
                }
            });
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn get_all_theme_configs(
        theme_holder: &themes::themes::ThemeHolder,
    ) -> Vec<crate::ThemeConfig> {
        let local_themes = Self::get_all_themes_list(theme_holder);
        let mut configs: Vec<crate::ThemeConfig> =
            local_themes.iter().map(Self::map_theme_to_config).collect();

        let local_ids: std::collections::HashSet<String> =
            local_themes.into_iter().map(|t| t.id).collect();

        for manifest in theme_holder.get_remote_manifests() {
            if !local_ids.contains(&manifest.id) {
                configs.push(Self::map_manifest_to_config(&manifest));
            }
        }

        // Pin default and current active themes to the top, then sort all
        // remaining presets alphabetically by name.
        configs.sort_by(|a, b| {
            if a.id == "default" {
                std::cmp::Ordering::Less
            } else if b.id == "default" {
                std::cmp::Ordering::Greater
            } else if a.id == "current" {
                std::cmp::Ordering::Less
            } else if b.id == "current" {
                std::cmp::Ordering::Greater
            } else {
                a.name.cmp(&b.name)
            }
        });

        configs
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn get_all_themes_list(
        theme_holder: &themes::themes::ThemeHolder,
    ) -> Vec<ThemeDetails> {
        let mut list = Vec::new();
        match theme_holder.load_all_themes() {
            Ok(themes) => {
                for (id, mut theme) in themes {
                    if id == "default" {
                        theme.id = "default".to_string();
                        theme.name = "Default".to_string();
                        theme.author = Some("Moosync".to_string());
                        theme.description = Some("System default theme".to_string());
                    }
                    list.push(theme);
                }
            }
            Err(e) => {
                tracing::error!("Failed to load all themes: {:?}", e);
            }
        }
        list.sort_by(|a, b| {
            if a.id == "default" {
                std::cmp::Ordering::Less
            } else if b.id == "default" {
                std::cmp::Ordering::Greater
            } else if a.id == "current" {
                std::cmp::Ordering::Less
            } else if b.id == "current" {
                std::cmp::Ordering::Greater
            } else {
                a.name.cmp(&b.name)
            }
        });
        list
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn map_theme_to_config(theme: &ThemeDetails) -> crate::ThemeConfig {
        build_theme_config(
            theme.id.clone(),
            theme.name.clone(),
            theme.description.clone().unwrap_or_default(),
            theme.author.clone().unwrap_or_default(),
            String::new(),
            theme.get_theme_item_or_default(),
        )
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn map_manifest_to_config(manifest: &ThemeManifest) -> crate::ThemeConfig {
        build_theme_config(
            manifest.id.clone(),
            manifest.name.clone(),
            manifest.description.clone().unwrap_or_default(),
            manifest.author.clone().unwrap_or_default(),
            manifest.registry.clone().unwrap_or_default(),
            manifest.theme.clone().unwrap_or_default(),
        )
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn apply_theme(main_window: &MainWindow, theme: &ThemeDetails) {
        let theme_global = main_window.global::<crate::Theme>();
        let theme_item = theme.get_theme_item_or_default();
        let default_item = types::prelude::get_default_theme_item();

        let mut all_keys = std::collections::HashSet::new();
        all_keys.extend(default_item.get_all_keys());
        all_keys.extend(theme_item.get_all_keys());

        for key in all_keys {
            let val = theme_item
                .get_constant(&key)
                .or_else(|| default_item.get_constant(&key))
                .unwrap_or_default();
            Self::apply_single_constant(&theme_global, &key, &val);
        }

        update_theme_constants_ui(main_window, theme);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn apply_single_constant(theme_global: &crate::Theme, name: &str, value: &str) {
        let set_color = |setter: &dyn Fn(slint::Color)| {
            if let Some(c) = parse_color(value) {
                setter(c);
            }
        };

        let set_length = |setter: &dyn Fn(f32)| {
            if let Some(l) = parse_length(value) {
                setter(l);
            }
        };

        #[cfg(not(target_os = "android"))]
        theme_macro::generate_theme_apply!(
            "ui/slint/src/constants.slint",
            theme_global,
            name,
            set_color,
            set_length
        );
        #[cfg(target_os = "android")]
        theme_macro::generate_theme_apply!(
            "ui/slint/src/constants_mobile.slint",
            theme_global,
            name,
            set_color,
            set_length
        );
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn flush_changes(
        state_manager: &StateManager,
        main_window_weak: &slint::Weak<MainWindow>,
        pending_changes: &mut std::collections::HashMap<String, String>,
    ) {
        let theme_holder = state_manager.get_theme_holder();
        let preference_config = state_manager.get_preference_config();

        let active_theme_id = preference_config
            .inner
            .load(&ACTIVE_THEME_ID)
            .value::<String>()
            .unwrap_or_else(|| "default".to_string());

        let mut target_theme_id = active_theme_id.clone();

        if active_theme_id != "current" {
            let active_theme = theme_holder
                .inner
                .load_theme(active_theme_id.clone())
                .unwrap_or_else(|_| ThemeDetails {
                    id: "default".to_string(),
                    name: "Default".to_string(),
                    ..Default::default()
                });

            let mut current_theme = active_theme.clone();
            current_theme.id = "current".to_string();
            current_theme.name = "Current".to_string();
            current_theme.author = Some("Me".to_string());
            current_theme.description = Some("Modified theme".to_string());

            if let Err(e) = theme_holder.inner.save_theme(current_theme) {
                tracing::error!("Failed to clone active theme to 'current': {:?}", e);
                return;
            }

            let mut pref_item = (*ACTIVE_THEME_ID).clone();
            pref_item.value = Some(PreferenceValue {
                value: Some(preference_value::Value::StringValue("current".to_string())),
            });
            let _ = preference_config.inner.save(pref_item);
            target_theme_id = "current".to_string();
        }

        match theme_holder.inner.load_theme(target_theme_id.clone()) {
            Ok(mut theme) => {
                let mut theme_item = theme
                    .theme
                    .clone()
                    .unwrap_or_else(types::prelude::get_default_theme_item);

                // Insert all pending changes
                for (name, val) in pending_changes.drain() {
                    theme_item.set_constant(&name, val);
                }
                theme.theme = Some(theme_item);

                if let Err(e) = theme_holder.inner.save_theme(theme.clone()) {
                    tracing::error!("Failed to save theme modifications to 'current': {:?}", e);
                    return;
                }

                let configs = Self::get_all_theme_configs(&theme_holder.inner);

                let _ = slint::invoke_from_event_loop({
                    let main_window_weak = main_window_weak.clone();
                    let target_theme_id = target_theme_id.clone();
                    move || {
                        if let Some(main_window) = main_window_weak.upgrade() {
                            let themes_props = main_window.global::<ThemesPageProps>();
                            themes_props.set_active_theme_id(target_theme_id.into());
                            Self::apply_theme(&main_window, &theme);

                            let vec_model = slint::VecModel::default();
                            for c in configs {
                                vec_model.push(c);
                            }
                            themes_props.set_available_themes(slint::ModelRc::new(vec_model));
                        }
                    }
                });
            }
            Err(e) => {
                tracing::error!("Failed to load theme '{}': {:?}", target_theme_id, e);
            }
        }
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn listen_active_theme_preference(
    state_manager: &'static StateManager,
    main_window_weak: slint::Weak<MainWindow>,
) {
    tokio::spawn(
        async move {
            let preference_config = state_manager.get_preference_config();
            preference_config
                .on_preference_changed
                .listen_filtered_immediate(
                    {
                        let main_window_weak = main_window_weak.clone();
                        move |key| {
                            if key == ACTIVE_THEME_ID.id.as_str() {
                                let main_window_weak = main_window_weak.clone();
                                tokio::spawn(
                                    async move {
                                        let theme_holder = state_manager.get_theme_holder();
                                        let preference_config =
                                            state_manager.get_preference_config();
                                        let active_theme_id = preference_config
                                            .inner
                                            .load(&ACTIVE_THEME_ID)
                                            .value::<String>()
                                            .unwrap_or_else(|| "default".to_string());

                                        let active_theme = theme_holder
                                            .inner
                                            .load_theme(active_theme_id.clone())
                                            .unwrap_or_else(|_| ThemeDetails {
                                                id: "default".to_string(),
                                                name: "Default".to_string(),
                                                ..Default::default()
                                            });

                                        let configs = ThemesPageHandler::get_all_theme_configs(
                                            &theme_holder.inner,
                                        );

                                        let _ = slint::invoke_from_event_loop(move || {
                                            if let Some(main_window) = main_window_weak.upgrade() {
                                                let themes_props =
                                                    main_window.global::<ThemesPageProps>();
                                                themes_props
                                                    .set_active_theme_id(active_theme_id.into());
                                                ThemesPageHandler::apply_theme(
                                                    &main_window,
                                                    &active_theme,
                                                );

                                                let vec_model = slint::VecModel::default();
                                                for c in configs {
                                                    vec_model.push(c);
                                                }
                                                themes_props.set_available_themes(
                                                    slint::ModelRc::new(vec_model),
                                                );
                                            }
                                        });
                                    }
                                    .in_current_span(),
                                );
                            }
                        }
                    },
                    ACTIVE_THEME_ID.id.as_str(),
                );
        }
        .in_current_span(),
    );
}

#[tracing::instrument(level = "debug", skip_all)]
async fn ensure_theme_installed(
    theme_id: &str,
    theme_holder: &themes::themes::ThemeHolder,
) -> Result<(), themes::error::ThemesError> {
    if theme_holder.load_theme(theme_id.to_string()).is_ok() {
        return Ok(());
    }

    let remotes = theme_holder.get_remote_manifests();
    if let Some(remote_manifest) = remotes.into_iter().find(|m| m.id == theme_id) {
        tracing::info!("Downloading and installing remote theme: {}", theme_id);
        theme_holder.install_remote_theme(&remote_manifest).await?;
    }
    Ok(())
}

#[tracing::instrument(level = "debug", skip_all)]
async fn select_preset_theme(
    theme_id: String,
    main_window_weak: slint::Weak<MainWindow>,
    state_manager: &'static StateManager,
) {
    tracing::debug!("Selecting preset theme: {}", theme_id);
    let theme_holder = state_manager.get_theme_holder();
    let preference_config = state_manager.get_preference_config();

    if let Err(e) = ensure_theme_installed(&theme_id, &theme_holder.inner).await {
        tracing::error!("Failed to install remote theme {}: {:?}", theme_id, e);
        return;
    }

    match theme_holder.inner.load_theme(theme_id.clone()) {
        Ok(theme) => {
            let mut pref_item = (*ACTIVE_THEME_ID).clone();
            pref_item.value = Some(PreferenceValue {
                value: Some(preference_value::Value::StringValue(theme_id.clone())),
            });
            if let Err(e) = preference_config.inner.save(pref_item) {
                tracing::error!("Failed to save active theme preference: {:?}", e);
            }
            theme_holder.inner.on_theme_changed.emit(theme.clone());

            let theme_id = theme_id.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(main_window) = main_window_weak.upgrade() {
                    main_window
                        .global::<ThemesPageProps>()
                        .set_active_theme_id(theme_id.into());
                    ThemesPageHandler::apply_theme(&main_window, &theme);
                }
            });
        }
        Err(e) => {
            tracing::error!("Failed to load theme {}: {:?}", theme_id, e);
        }
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn setup_select_preset_theme(main_window: &MainWindow, state_manager: &'static StateManager) {
    main_window
        .global::<crate::AppCallbacks>()
        .on_select_preset_theme({
            let main_window_weak = main_window.as_weak();
            move |theme_id| {
                tokio::spawn(
                    select_preset_theme(
                        theme_id.to_string(),
                        main_window_weak.clone(),
                        state_manager,
                    )
                    .instrument(tracing::debug_span!("slint_cb_on_select_preset_theme")),
                );
            }
        });
}

#[tracing::instrument(level = "debug", skip_all)]
fn setup_theme_constant_changed(main_window: &MainWindow, state_manager: &'static StateManager) {
    let debounce_state = Arc::new(Mutex::new((
        HashMap::<String, String>::new(),
        Option::<tokio::task::JoinHandle<()>>::None,
    )));

    main_window
        .global::<crate::AppCallbacks>()
        .on_theme_constant_changed({
            let main_window_weak = main_window.as_weak();
            let debounce_state = debounce_state.clone();
            move |constant_name, value| {
                let constant_name = constant_name.to_string();
                let value = value.to_string();
                tracing::info!(
                    "theme_constant_changed callback: name={}, value={}",
                    constant_name,
                    value
                );

                if let Some(main_window) = main_window_weak.upgrade() {
                    let theme_global = main_window.global::<crate::Theme>();
                    ThemesPageHandler::apply_single_constant(&theme_global, &constant_name, &value);
                }

                let mut guard = debounce_state.lock().unwrap();
                guard.0.insert(constant_name, value);
                if let Some(handle) = guard.1.take() {
                    handle.abort();
                }

                let main_window_weak = main_window_weak.clone();
                let debounce_state_clone = debounce_state.clone();
                guard.1 = Some(tokio::spawn(
                    async move {
                        tokio::time::sleep(Duration::from_millis(150)).await;
                        let mut pending = {
                            let mut guard = debounce_state_clone.lock().unwrap();
                            guard.1 = None;
                            mem::take(&mut guard.0)
                        };
                        if !pending.is_empty() {
                            ThemesPageHandler::flush_changes(
                                state_manager,
                                &main_window_weak,
                                &mut pending,
                            );
                        }
                    }
                    .in_current_span(),
                ));
            }
        });
}

#[tracing::instrument(level = "debug", skip_all)]
async fn save_custom_theme(
    state_manager: &'static StateManager,
    main_window_weak: slint::Weak<MainWindow>,
    name: String,
    author: String,
    description: String,
) {
    tracing::debug!("Saving custom theme: name={}, author={}", name, author);
    let theme_holder = state_manager.get_theme_holder();
    let preference_config = state_manager.get_preference_config();

    let mut theme = match theme_holder.inner.load_theme("current".to_string()) {
        Ok(theme) => theme,
        Err(e) => {
            tracing::error!(
                "Failed to load 'current' theme for saving custom theme: {:?}",
                e
            );
            return;
        }
    };

    let new_id = uuid::Uuid::new_v4().to_string();
    theme.id = new_id.clone();
    theme.name = name;
    theme.author = Some(author);
    theme.description = Some(description);

    if let Err(e) = theme_holder.inner.save_theme(theme.clone()) {
        tracing::error!("Failed to save custom theme: {:?}", e);
        return;
    }

    if let Err(e) = theme_holder.inner.remove_theme("current".to_string()) {
        tracing::error!("Failed to remove current temporary theme: {:?}", e);
    }
    let mut pref_item = (*ACTIVE_THEME_ID).clone();
    pref_item.value = Some(PreferenceValue {
        value: Some(preference_value::Value::StringValue(new_id.clone())),
    });
    if let Err(e) = preference_config.inner.save(pref_item) {
        tracing::error!("Failed to save active theme preference: {:?}", e);
    }

    let configs = ThemesPageHandler::get_all_theme_configs(&theme_holder.inner);

    let _ = slint::invoke_from_event_loop(move || {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };
        let themes_props = main_window.global::<ThemesPageProps>();
        themes_props.set_active_theme_id(new_id.into());
        ThemesPageHandler::apply_theme(&main_window, &theme);

        let vec_model = slint::VecModel::default();
        for c in configs {
            vec_model.push(c);
        }
        themes_props.set_available_themes(slint::ModelRc::new(vec_model));
    });
}

#[tracing::instrument(level = "debug", skip_all)]
fn setup_save_custom_theme(main_window: &MainWindow, state_manager: &'static StateManager) {
    main_window
        .global::<crate::AppCallbacks>()
        .on_save_custom_theme({
            let main_window_weak = main_window.as_weak();
            move |name, author, description| {
                tokio::spawn(
                    save_custom_theme(
                        state_manager,
                        main_window_weak.clone(),
                        name.to_string(),
                        author.to_string(),
                        description.to_string(),
                    )
                    .instrument(tracing::debug_span!("slint_cb_on_save_custom_theme")),
                );
            }
        });
}

#[tracing::instrument(level = "debug", skip_all)]
fn update_available_themes(
    main_window_weak: slint::Weak<MainWindow>,
    state_manager: &'static StateManager,
) {
    let theme_holder = state_manager.get_theme_holder();
    let configs = ThemesPageHandler::get_all_theme_configs(&theme_holder.inner);
    let update_ui = move |main_window: &MainWindow| {
        let vec_model = VecModel::default();
        for c in configs {
            vec_model.push(c);
        }
        main_window
            .global::<ThemesPageProps>()
            .set_available_themes(ModelRc::new(vec_model));
    };

    if let Some(main_window) = main_window_weak.upgrade() {
        update_ui(&main_window);
    } else {
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(main_window) = main_window_weak.upgrade() {
                update_ui(&main_window);
            }
        });
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn handle_theme_changed(
    changed_theme: ThemeDetails,
    main_window_weak: slint::Weak<MainWindow>,
    state_manager: &'static StateManager,
) {
    let preference_config = state_manager.get_preference_config();
    let active_theme_id = preference_config
        .inner
        .load(&ACTIVE_THEME_ID)
        .value::<String>()
        .unwrap_or_else(|| "default".to_string());

    if changed_theme.id == active_theme_id {
        let changed_theme = changed_theme.clone();
        let mw_weak = main_window_weak.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(main_window) = mw_weak.upgrade() {
                ThemesPageHandler::apply_theme(&main_window, &changed_theme);
            }
        });
    }

    update_available_themes(main_window_weak, state_manager);
}

#[tracing::instrument(level = "debug", skip_all)]
fn setup_themes_updated_listener(
    main_window_weak: slint::Weak<MainWindow>,
    state_manager: &'static StateManager,
) -> types::subscription::CancelHandle {
    let theme_holder = state_manager.get_theme_holder();
    theme_holder.on_themes_updated.listen(move |_| {
        let main_window_weak = main_window_weak.clone();
        tokio::spawn(
            async move {
                update_available_themes(main_window_weak.clone(), state_manager);
                ThemesPageHandler::refresh_preferences(main_window_weak, state_manager);
            }
            .in_current_span(),
        );
    })
}

#[tracing::instrument(level = "debug", skip_all)]
fn listen_theme_holder_changed(
    state_manager: &'static StateManager,
    main_window_weak: slint::Weak<MainWindow>,
) {
    tokio::spawn({
        async move {
            let theme_holder = state_manager.get_theme_holder();
            let mw_weak1 = main_window_weak.clone();
            let _cancel_theme_changed =
                theme_holder.on_theme_changed.listen(move |changed_theme| {
                    let changed_theme = changed_theme.clone();
                    let mw_weak = mw_weak1.clone();
                    tokio::spawn(
                        async move {
                            handle_theme_changed(changed_theme, mw_weak, state_manager);
                        }
                        .in_current_span(),
                    );
                });

            let _cancel_themes_updated =
                setup_themes_updated_listener(main_window_weak, state_manager);
        }
        .in_current_span()
    });
}

impl PageHandler for ThemesPageHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) {
        let main_window_weak = self.main_window.as_weak();

        ThemesPageHandler::refresh_preferences(main_window_weak.clone(), self.state_manager);
        listen_active_theme_preference(self.state_manager, main_window_weak.clone());
        setup_select_preset_theme(self.main_window, self.state_manager);
        setup_theme_constant_changed(self.main_window, self.state_manager);
        setup_save_custom_theme(self.main_window, self.state_manager);
        listen_theme_holder_changed(self.state_manager, main_window_weak);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) {
        tracing::info!("ThemesPageHandler: on_show");
        let state_manager = self.state_manager;
        let main_window_weak = self.main_window.as_weak();
        tokio::spawn(
            async move {
                update_available_themes(main_window_weak.clone(), state_manager);
                ThemesPageHandler::refresh_preferences(main_window_weak, state_manager);
            }
            .in_current_span(),
        );
    }
}
