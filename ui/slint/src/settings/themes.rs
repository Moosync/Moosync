// Moosync
// Copyright (C) 2024, 2025  Moosync <support@moosync.app>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//

use slint::ComponentHandle;
use state_manager::StateManager;
use themes_proto::moosync::types::ThemeDetails;
use types::prelude::{ThemeExt, ThemeItemExt};

use crate::{MainWindow, pages::PageHandler};

theme_macro::generate_theme_ui_helpers!("ui/slint/src/constants.slint");

pub struct ThemesPageHandler<'a> {
    main_window: &'a MainWindow,
    state_manager: &'a StateManager,
}

impl<'a> ThemesPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &'a MainWindow, state_manager: &'a StateManager) -> Self {
        Self {
            main_window,
            state_manager,
        }
    }
}

impl<'a> PageHandler for ThemesPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) {
        let state_manager = self.state_manager.clone();
        let main_window_weak = self.main_window.as_weak();

        // debounce theme configuration writes to disk
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<(String, String)>();
        let state_manager_bg = state_manager.clone();
        let main_window_weak_bg = main_window_weak.clone();

        tokio::spawn(async move {
            let mut pending_changes = std::collections::HashMap::new();
            loop {
                let timeout = tokio::time::sleep(std::time::Duration::from_millis(150));
                tokio::pin!(timeout);

                tokio::select! {
                    maybe_msg = rx.recv() => {
                        if let Some((name, val)) = maybe_msg {
                            pending_changes.insert(name, val);
                        } else {
                            if !pending_changes.is_empty() {
                                flush_changes(&state_manager_bg, &main_window_weak_bg, &mut pending_changes).await;
                            }
                            break;
                        }
                    }
                    _ = &mut timeout => {
                        if !pending_changes.is_empty() {
                            flush_changes(&state_manager_bg, &main_window_weak_bg, &mut pending_changes).await;
                        }
                    }
                }
            }
        });

        tokio::spawn(async move {
            let preference_config = state_manager.get_preference_config().await;
            preference_config.on_preference_changed_immediate(
                {
                    let main_window_weak = main_window_weak.clone();
                    let state_manager = state_manager.clone();
                    move |key| {
                        if key == preferences::keys::ActiveThemeId {
                            let state_manager = state_manager.clone();
                            let main_window_weak = main_window_weak.clone();
                            tokio::spawn(async move {
                                let theme_holder = state_manager.get_theme_holder().await;
                                let preference_config = state_manager.get_preference_config().await;
                                let active_theme_id = preference_config
                                    .inner
                                    .load(preferences::keys::ActiveThemeId)
                                    .unwrap_or_else(|_| "default".to_string());

                                let active_theme = theme_holder
                                    .inner
                                    .load_theme(active_theme_id.clone())
                                    .unwrap_or_else(|_| {
                                        let mut def = ThemeDetails::default();
                                        def.id = "default".to_string();
                                        def.name = "Default".to_string();
                                        def
                                    });

                                let themes_list = get_all_themes_list(&theme_holder.inner);

                                let _ = slint::invoke_from_event_loop(move || {
                                    if let Some(main_window) = main_window_weak.upgrade() {
                                        main_window.set_active_theme_id(active_theme_id.into());
                                        apply_theme(&main_window, &active_theme);

                                        let vec_model = slint::VecModel::default();
                                        for t in themes_list {
                                            vec_model.push(map_theme_to_config(&t));
                                        }
                                        main_window
                                            .set_available_themes(slint::ModelRc::new(vec_model));
                                    }
                                });
                            });
                        }
                    }
                },
                preferences::keys::ActiveThemeId,
            );
        });

        self.main_window
            .global::<crate::AppCallbacks>()
            .on_select_preset_theme({
                let state_manager = self.state_manager.clone();
                let main_window_weak = self.main_window.as_weak();
                move |theme_id| {
                    let theme_id = theme_id.to_string();
                    let state_manager = state_manager.clone();
                    let main_window_weak = main_window_weak.clone();

                    tokio::spawn(async move {
                        let theme_holder = state_manager.get_theme_holder().await;
                        let preference_config = state_manager.get_preference_config().await;

                        if let Ok(theme) = theme_holder.inner.load_theme(theme_id.clone()) {
                            let _ = preference_config
                                .inner
                                .save(preferences::keys::ActiveThemeId, theme_id.clone());
                            theme_holder.inner.on_theme_changed.run_all(|cb| cb(&theme));

                            let theme_id = theme_id.clone();
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(main_window) = main_window_weak.upgrade() {
                                    main_window.set_active_theme_id(theme_id.into());
                                    apply_theme(&main_window, &theme);
                                }
                            });
                        }
                    });
                }
            });

        self.main_window
            .global::<crate::AppCallbacks>()
            .on_theme_constant_changed({
                let main_window_weak = self.main_window.as_weak();
                let tx = tx.clone();
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
                        apply_single_constant(&theme_global, &constant_name, &value);
                    }

                    let _ = tx.send((constant_name, value));
                }
            });

        self.main_window
            .global::<crate::AppCallbacks>()
            .on_save_custom_theme({
                let state_manager = self.state_manager.clone();
                let main_window_weak = self.main_window.as_weak();
                move |name, author, description| {
                    let name = name.to_string();
                    let author = author.to_string();
                    let description = description.to_string();
                    let state_manager = state_manager.clone();
                    let main_window_weak = main_window_weak.clone();

                    tokio::spawn(async move {
                        let theme_holder = state_manager.get_theme_holder().await;
                        let preference_config = state_manager.get_preference_config().await;

                        if let Ok(mut theme) = theme_holder.inner.load_theme("current".to_string())
                        {
                            let new_id = uuid::Uuid::new_v4().to_string();
                            theme.id = new_id.clone();
                            theme.name = name;
                            theme.author = Some(author);
                            theme.description = Some(description);

                            if let Err(e) = theme_holder.inner.save_theme(theme.clone()) {
                                tracing::error!("Failed to save custom theme: {:?}", e);
                                return;
                            }

                            let _ = theme_holder.inner.remove_theme("current".to_string());
                            let _ = preference_config
                                .inner
                                .save(preferences::keys::ActiveThemeId, new_id.clone());

                            let themes_list = get_all_themes_list(&theme_holder.inner);

                            let main_window_weak = main_window_weak.clone();
                            let new_id = new_id.clone();
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(main_window) = main_window_weak.upgrade() {
                                    main_window.set_active_theme_id(new_id.into());
                                    apply_theme(&main_window, &theme);

                                    let vec_model = slint::VecModel::default();
                                    for t in themes_list {
                                        vec_model.push(map_theme_to_config(&t));
                                    }
                                    main_window
                                        .set_available_themes(slint::ModelRc::new(vec_model));
                                }
                            });
                        }
                    });
                }
            });

        tokio::spawn({
            let state_manager = self.state_manager.clone();
            let main_window_weak = self.main_window.as_weak();
            async move {
                let theme_holder = state_manager.get_theme_holder().await;
                let main_window_weak = main_window_weak.clone();
                let state_manager = state_manager.clone();

                theme_holder.on_theme_changed(move |changed_theme| {
                    let changed_theme = changed_theme.clone();
                    let main_window_weak = main_window_weak.clone();
                    let state_manager = state_manager.clone();

                    tokio::spawn(async move {
                        let theme_holder = state_manager.get_theme_holder().await;
                        let preference_config = state_manager.get_preference_config().await;

                        let active_theme_id = preference_config
                            .inner
                            .load(preferences::keys::ActiveThemeId)
                            .unwrap_or_else(|_| "default".to_string());

                        if changed_theme.id == active_theme_id {
                            let changed_theme = changed_theme.clone();
                            let main_window_weak = main_window_weak.clone();
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(main_window) = main_window_weak.upgrade() {
                                    apply_theme(&main_window, &changed_theme);
                                }
                            });
                        }

                        let themes_list = get_all_themes_list(&theme_holder.inner);
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(main_window) = main_window_weak.upgrade() {
                                let vec_model = slint::VecModel::default();
                                for t in themes_list {
                                    vec_model.push(map_theme_to_config(&t));
                                }
                                main_window.set_available_themes(slint::ModelRc::new(vec_model));
                            }
                        });
                    });
                });
            }
        });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) {}
    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) {}
}

#[tracing::instrument(level = "debug", skip_all)]
fn get_all_themes_list(theme_holder: &themes::themes::ThemeHolder) -> Vec<ThemeDetails> {
    let mut list = Vec::new();
    if let Ok(themes) = theme_holder.load_all_themes() {
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
fn parse_color(val: &str) -> Option<slint::Color> {
    let val = val.trim();
    if val.starts_with('#') {
        let hex = &val[1..];
        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(slint::Color::from_rgb_u8(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(slint::Color::from_argb_u8(a, r, g, b))
            }
            _ => None,
        }
    } else if val.starts_with("rgb") {
        let start = val.find('(')? + 1;
        let end = val.rfind(')')?;
        let parts: Vec<&str> = val[start..end].split(',').map(|s| s.trim()).collect();
        if parts.len() >= 3 {
            let r = parts[0].parse::<f32>().ok()? as u8;
            let g = parts[1].parse::<f32>().ok()? as u8;
            let b = parts[2].parse::<f32>().ok()? as u8;
            if parts.len() == 4 {
                let a = parts[3].parse::<f32>().ok()?;
                Some(slint::Color::from_argb_f32(
                    a,
                    r as f32 / 255.0,
                    g as f32 / 255.0,
                    b as f32 / 255.0,
                ))
            } else {
                Some(slint::Color::from_rgb_u8(r, g, b))
            }
        } else {
            None
        }
    } else {
        None
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn parse_length(val: &str) -> Option<f32> {
    let val = val.trim();
    if val.ends_with("px") {
        val[..val.len() - 2].parse::<f32>().ok()
    } else {
        val.parse::<f32>().ok()
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn map_theme_to_config(theme: &ThemeDetails) -> crate::ThemeConfig {
    let theme_item = theme.get_theme_item_or_default();
    let default_item = types::prelude::get_default_theme_item();

    let get_color = |key: &str| -> crate::RgbaColor {
        let val = theme_item
            .get_constant(key)
            .or_else(|| default_item.get_constant(key))
            .unwrap_or_default();
        if let Some(color) = parse_color(&val) {
            crate::RgbaColor {
                r: color.red() as f32,
                g: color.green() as f32,
                b: color.blue() as f32,
                a: color.alpha() as f32 / 255.0,
            }
        } else {
            crate::RgbaColor {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }
        }
    };

    let get_length = |key: &str| -> f32 {
        let val = theme_item
            .get_constant(key)
            .or_else(|| default_item.get_constant(key))
            .unwrap_or_default();
        parse_length(&val).unwrap_or(0.0)
    };

    crate::ThemeConfig {
        id: theme.id.clone().into(),
        name: theme.name.clone().into(),
        description: theme.description.clone().unwrap_or_default().into(),
        author: theme.author.clone().unwrap_or_default().into(),
        preview_bg: get_color("tertiary"),
        accent_color: get_color("accent"),
        primary_color: get_color("primary"),
        secondary_color: get_color("secondary"),
        border_radius: get_length("borderRadiusLg"),
        border_width: get_length("borderWidth"),
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn apply_theme(main_window: &MainWindow, theme: &ThemeDetails) {
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
        apply_single_constant(&theme_global, &key, &val);
    }

    update_theme_constants_ui(main_window, theme);
}

#[tracing::instrument(level = "debug", skip_all)]
fn apply_single_constant(theme_global: &crate::Theme, name: &str, value: &str) {
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

    theme_macro::generate_theme_apply!(
        "ui/slint/src/constants.slint",
        theme_global,
        name,
        set_color,
        set_length
    );
}

#[tracing::instrument(level = "debug", skip_all)]
async fn flush_changes(
    state_manager: &StateManager,
    main_window_weak: &slint::Weak<MainWindow>,
    pending_changes: &mut std::collections::HashMap<String, String>,
) {
    let theme_holder = state_manager.get_theme_holder().await;
    let preference_config = state_manager.get_preference_config().await;

    let active_theme_id = preference_config
        .inner
        .load(preferences::keys::ActiveThemeId)
        .unwrap_or_else(|_| "default".to_string());

    let mut target_theme_id = active_theme_id.clone();

    if active_theme_id != "current" {
        let active_theme = theme_holder
            .inner
            .load_theme(active_theme_id.clone())
            .unwrap_or_else(|_| {
                let mut def = ThemeDetails::default();
                def.id = "default".to_string();
                def.name = "Default".to_string();
                def
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

        let _ = preference_config
            .inner
            .save(preferences::keys::ActiveThemeId, "current".to_string());
        target_theme_id = "current".to_string();
    }

    if let Ok(mut theme) = theme_holder.inner.load_theme(target_theme_id.clone()) {
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

        let themes_list = get_all_themes_list(&theme_holder.inner);

        let _ = slint::invoke_from_event_loop({
            let main_window_weak = main_window_weak.clone();
            let target_theme_id = target_theme_id.clone();
            move || {
                if let Some(main_window) = main_window_weak.upgrade() {
                    main_window.set_active_theme_id(target_theme_id.into());
                    apply_theme(&main_window, &theme);

                    let vec_model = slint::VecModel::default();
                    for t in themes_list {
                        vec_model.push(map_theme_to_config(&t));
                    }
                    main_window.set_available_themes(slint::ModelRc::new(vec_model));
                }
            }
        });
    }
}
