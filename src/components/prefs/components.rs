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

use std::{collections::HashMap, sync::Arc};

use leptos::{component, prelude::*, reactive::wrappers::write::SignalSetter, view, IntoView};
use leptos_i18n::t;
use leptos_use::use_debounce_fn_with_arg;
use types::{
    preferences::{CheckboxItems, CheckboxPreference, InputType},
    themes::ThemeDetails,
    ui::{extensions::ExtensionDetail, themes::ThemeModalState},
    window::DialogFilter,
};
use wasm_bindgen_futures::spawn_local;

use crate::{
    i18n::use_i18n,
    icons::{
        folder_icon::FolderIcon, new_theme_icon::NewThemeIcon, theme_view_icon::ThemeViewIcon,
        tooltip::Tooltip,
    },
    store::{
        modal_store::{ModalStore, Modals},
        ui_store::UiStore,
    },
    utils::{
        context_menu::{create_context_menu, ThemesContextMenu},
        invoke::{get_installed_extensions, load_all_themes, remove_extension},
        prefs::{
            load_secure, load_selective, open_file_browser, open_file_browser_single, save_secure,
            save_selective, save_selective_number,
        },
    },
};

#[tracing::instrument(level = "debug", skip(key, title, tooltip))]
#[component]
pub fn PathsPref<K, H, K1, H1>(
    #[prop()] key: String,
    #[prop()] title: K,
    #[prop()] tooltip: K1,
    #[prop()] mobile: bool,
) -> impl IntoView
where
    K: Fn() -> H + Send + Sync + 'static,
    H: IntoView + 'static,
    K1: Fn() -> H1 + Send + Sync + 'static,
    H1: IntoView + 'static,
{
    let ui_store = expect_context::<RwSignal<UiStore>>();
    let is_mobile = create_read_slice(ui_store, |u| u.get_is_mobile()).get();
    if is_mobile && !mobile {
        return ().into_any();
    }

    let should_write = RwSignal::new(false);
    let paths: RwSignal<Vec<String>> = RwSignal::new(vec![]);
    load_selective(key.clone(), paths.write_only());
    let selected_paths = RwSignal::new(vec![]);
    Effect::new(move || {
        let new_paths = selected_paths.get();
        tracing::debug!("Got new paths: {:?}", new_paths);
        if !new_paths.is_empty() {
            paths.update(|paths| paths.extend(new_paths.iter().cloned()));
        }
    });

    let start_scan = move |_| {
        spawn_local(async move {
            let _ = crate::utils::invoke::start_scan(None).await;
        })
    };

    let key_clone = key.clone();
    Effect::new(move || {
        let value = paths.get();
        tracing::debug!("Should write {}, {:?}", should_write.get_untracked(), value);
        if !should_write.get_untracked() {
            untrack(|| should_write.set(true));
            return;
        }
        tracing::debug!("Saving paths: {:?}", value);
        save_selective(key_clone.clone(), value);
    });
    let i18n = use_i18n();
    view! {
        <div class="container-fluid mt-4">
            <div class="row no-gutters align-items-center">
                <div class="row no-gutters">
                    <div class="col-auto align-self-center title d-flex preference-title">
                        {title()}
                    </div>
                    <div class="col-auto ml-2">
                        <Tooltip>{tooltip()}</Tooltip>
                    </div>
                </div>
                <div class="col-auto new-directories ml-auto justify-content-center">
                    <div on:click=start_scan>{"Refresh"}</div>
                </div>
                <div class="col-auto new-directories ml-4">
                    <div
                        class="add-directories-button"
                        on:click=move |_| open_file_browser(true, true, vec![], selected_paths)
                    >
                        {t!(i18n, settings.paths.add_folder)}
                    </div>
                </div>
            </div>
            <div class="row no-gutters path-prefs-background w-100 mt-2 d-flex">
                <For
                    each=move || paths.get()
                    key=|p| p.clone()
                    children=move |path: String| {
                        view! {
                            <div class="row no-gutters mt-3 item w-100">
                                <div class="col col-md-8 col-lg-9 align-self-center justify-content-start ml-3 no-checkbox-margin">
                                    <div class="item-text text-truncate">{path.clone()}</div>
                                </div>
                                <div class="col-auto align-self-center ml-auto">
                                    <div
                                        class="remove-button w-100"
                                        on:click=move |_| {
                                            paths.update(|paths| paths.retain(|p| *p != path))
                                        }
                                    >
                                        {t!(i18n, settings.paths.remove)}
                                    </div>
                                </div>
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }.into_any()
}

#[tracing::instrument(level = "debug", skip(key, title, tooltip, show_input, inp_type))]
#[component()]
pub fn InputPref<K, H, K1, H1>(
    #[prop()] key: String,
    #[prop()] title: K,
    #[prop()] tooltip: K1,
    #[prop()] show_input: bool,
    #[prop()] inp_type: String,
    #[prop()] mobile: bool,
    #[prop(default = false)] is_secure: bool,
) -> impl IntoView
where
    K: Fn() -> H + Send + Sync + 'static,
    H: IntoView + 'static,
    K1: Fn() -> H1 + Send + Sync + 'static,
    H1: IntoView + 'static,
{
    let load_fn: fn(String, SignalSetter<String>) = if is_secure {
        load_secure::<String>
    } else {
        load_selective::<String>
    };
    let save_fn: fn(String, String) = if is_secure {
        save_secure::<String>
    } else {
        save_selective::<String>
    };

    let ui_store = expect_context::<RwSignal<UiStore>>();
    let is_mobile = create_read_slice(ui_store, |u| u.get_is_mobile()).get();
    if is_mobile && !mobile {
        return ().into_any();
    }

    let should_write = RwSignal::new(false);
    let pref_value = RwSignal::new(Default::default());
    let pref_key = key.clone();

    if inp_type == "number" {
        let num_pref = RwSignal::new(f64::default());
        load_selective(pref_key.clone(), num_pref.write_only());
        Effect::new(move || {
            let num_pref = num_pref.get();
            pref_value.set(format!("{num_pref}"));
        });
    } else {
        load_fn(pref_key.clone(), pref_value.write_only().into());
    }
    let inp_type_clone = inp_type.clone();
    Effect::new(move || {
        let value = pref_value.get();
        if !should_write.get_untracked() {
            untrack(|| should_write.set(true));
            return;
        }

        tracing::debug!("Input type - {}", inp_type_clone);
        if inp_type_clone.clone() == "number" {
            save_selective_number(pref_key.clone(), value);
        } else {
            save_fn(pref_key.clone(), value);
        }
    });

    let inp_type_clone = inp_type.clone();
    let debounced_update = use_debounce_fn_with_arg(
        move |event: web_sys::Event| {
            let value = event_target_value(&event);
            if inp_type_clone.clone() == "number" && value.parse::<f64>().is_err() {
                tracing::debug!("Invalid number");
                return;
            }
            pref_value.set(value);
        },
        500.0,
    );
    view! {
        <div class="container-fluid  mt-4">
            <div class="row no-gutters">
                <div class="col-auto align-self-center title d-flex preference-title">
                    {title()}
                </div>
                <div class="col-auto ml-2">
                    <Tooltip>{tooltip()}</Tooltip>
                </div>
            </div>
            <div class="row no-gutters input-prefs-background w-100 mt-2 d-flex align-content-center">
                {if !show_input {
                    view! {
                        <div class="col-auto align-self-center ml-4 folder-icon">
                            <FolderIcon on:click=move |_| open_file_browser_single(
                                true,
                                vec![],
                                pref_value,
                            ) />
                        </div>
                    }
                        .into_any()
                } else {
                    ().into_any()
                }}
                <div class="col-auto ml-3 mr-3 h-100 align-self-center flex-grow-1 d-flex">
                    {if show_input {
                        view! {
                            <input
                                class="ext-input w-100 ext-input-hover"
                                type=inp_type
                                prop:value=pref_value
                                on:input=move |e| {
                                    debounced_update(e);
                                }
                            />
                        }
                            .into_any()
                    } else {
                        view! {
                            <div class="item-text text-truncate file-picker-text align-self-center">
                                {move || pref_value.get()}
                            </div>
                        }
                            .into_any()
                    }}
                </div>
            </div>
        </div>
    }.into_any()
}

#[tracing::instrument(level = "debug", skip(key, title, tooltip, items, single))]
#[component()]
pub fn CheckboxPref<K, H, K1, H1>(
    #[prop()] key: String,
    #[prop()] title: K,
    #[prop()] tooltip: K1,
    #[prop()] items: Vec<CheckboxItems>,
    #[prop()] single: bool,
    #[prop()] mobile: bool,
) -> impl IntoView
where
    K: Fn() -> H + Send + Sync + 'static,
    H: IntoView + 'static,
    K1: Fn() -> H1 + Send + Sync + 'static,
    H1: IntoView + 'static,
{
    let ui_store = expect_context::<RwSignal<UiStore>>();
    let is_mobile = create_read_slice(ui_store, |u| u.get_is_mobile()).get();
    if is_mobile && !mobile {
        return ().into_any();
    }

    let should_write = RwSignal::new(false);
    let pref_value = RwSignal::<Vec<CheckboxPreference>>::new(Default::default());
    let pref_key = key;
    let pref_key_clone = pref_key.clone();
    load_selective(pref_key.clone(), pref_value.write_only());
    let last_enabled = RwSignal::new(String::new());
    Effect::new(move || {
        let value = pref_value.get();
        if !should_write.get_untracked() {
            should_write.update_untracked(|v| {
                *v = true;
            });
            return;
        }

        save_selective(pref_key.clone(), value.clone());
    });
    view! {
        <div class="container-fluid mt-4">
            <div class="row no-gutters">
                <div class="col-auto align-self-center title d-flex preference-title">
                    {title()}
                </div>
                <div class="col-auto ml-2">
                    <Tooltip>{tooltip()}</Tooltip>
                </div>
            </div>

            <For
                each=move || items.clone()
                key=|p| p.key.clone()
                children=move |item| {
                    let item_key_clone = item.key.clone();
                    let item_key_clone_1 = item_key_clone.clone();
                    let pref_key = pref_key_clone.clone();
                    view! {
                        <div class="row no-gutters item w-100 flex-nowrap">
                            <div class="col-auto align-self-center">
                                <div class="custom-control custom-checkbox">
                                    <input
                                        type=move || if single { "radio" } else { "checkbox" }
                                        class="custom-control-input"
                                        id=format!(
                                            "checkbox-{}-{}",
                                            pref_key.clone(),
                                            item_key_clone.clone(),
                                        )
                                        name=pref_key.clone()
                                        prop:checked=move || {
                                            pref_value
                                                .get()
                                                .iter()
                                                .find(|val| val.key == item_key_clone.clone())
                                                .map(|item| item.enabled)
                                                .unwrap_or(false)
                                        }
                                        on:change=move |ev| {
                                            let enabled = event_target_checked(&ev);
                                            tracing::info!("values changed");
                                            if single {
                                                pref_value
                                                    .update_untracked(|val| {
                                                        val.iter_mut()
                                                            .for_each(|v| {
                                                                v.enabled = false;
                                                            });
                                                    })
                                            }
                                            pref_value
                                                .update(|val| {
                                                    let found = val
                                                        .iter_mut()
                                                        .find(|val| val.key == item.key.clone());
                                                    if let Some(item) = found {
                                                        item.enabled = enabled;
                                                    } else {
                                                        val.push(CheckboxPreference {
                                                            key: item.key.clone(),
                                                            enabled,
                                                        });
                                                    }
                                                });
                                            if enabled {
                                                last_enabled.set(item.key.clone());
                                            }
                                        }
                                    />
                                    <label
                                        for=format!(
                                            "checkbox-{}-{}",
                                            pref_key.clone(),
                                            item_key_clone_1.clone(),
                                        )
                                        class="custom-control-label"
                                    ></label>
                                </div>
                            </div>
                            <div class="col-md-8 col-lg-9 col align-self-center ml-3 justify-content-start">
                                <div class="item-text text-truncate">{item.title}</div>
                            </div>
                        </div>
                    }
                }
            />
        </div>
    }.into_any()
}

#[tracing::instrument(level = "debug", skip(key, title, tooltip))]
#[component]
pub fn ThemesPref<K, H, K1, H1>(
    #[prop()] key: String,
    #[prop()] title: K,
    #[prop()] tooltip: K1,
) -> impl IntoView
where
    K: Fn() -> H + Send + Sync + 'static,
    H: IntoView + Copy + 'static,
    K1: Fn() -> H1 + Send + Sync + 'static,
    H1: IntoView + Copy + 'static,
{
    let all_themes: RwSignal<HashMap<String, ThemeDetails>> = RwSignal::new(Default::default());
    let load_themes = move || {
        spawn_local(async move {
            let themes = load_all_themes().await.unwrap();
            all_themes.set(themes);
        })
    };
    load_themes();
    let active_themes = RwSignal::new(vec![]);
    let active_theme_id = RwSignal::new(String::new());
    load_selective(key, active_theme_id);

    let context_menu = create_context_menu(ThemesContextMenu {
        id: None,
        refresh_cb: Arc::new(Box::new(load_themes)),
    });

    let modal_store: RwSignal<ModalStore> = expect_context();
    let render_themes = move || {
        let mut views = vec![];
        let active_theme_id = active_theme_id.get();
        for (key, theme) in all_themes.get() {
            let signal = RwSignal::new(key == active_theme_id);
            active_themes.update(|at| at.push(signal));
            let context_menu = context_menu.clone();
            let key_clone = key.clone();
            views.push(
                view! {
                    <div class="col-xl-3 col-5 p-2">
                        <div
                            class="theme-component-container"
                            on:contextmenu=move |ev| {
                                ev.prevent_default();
                                let context_menu = context_menu.clone();
                                let mut data = context_menu.get_data();
                                data.id = Some(key_clone.clone());
                                drop(data);
                                context_menu.show(ev);
                            }
                            on:click=move |_| {
                                active_themes
                                    .update(|at| {
                                        for s in at.iter() {
                                            s.set(false);
                                        }
                                        signal.set(true);
                                    });
                                let theme_id = key.clone();
                                tracing::debug!("Setting active theme: {}", theme_id);
                                save_selective("themes.active_theme".into(), theme_id);
                            }
                        >
                            <ThemeViewIcon
                                active=signal.read_only()
                                theme=Box::new(theme.clone())
                            />
                            <div class="theme-title-text">{theme.name}</div>
                            <div class="theme-author">{theme.author}</div>
                        </div>
                    </div>
                }
                .into_any(),
            );
        }

        views.push(
            view! {
                <div
                    class="col-xl-3 col-5 p-2"
                    on:click=move |_| {
                        modal_store
                            .update(|m| {
                                m.set_active_modal(
                                    Modals::ThemeModal(Box::new(ThemeModalState::None)),
                                );
                                m.on_modal_close(load_themes);
                            })
                    }
                >
                    <div class="theme-component-container">
                        <NewThemeIcon />
                        <div class="theme-title-text">{"Discover themes"}</div>
                    </div>
                </div>
            }
            .into_any(),
        );
        views.collect_view()
    };
    view! {
        <div class="container-fluid">
            <div class="row no-gutters">
                <div class="col-auto align-self-center title d-flex preference-title">
                    {title()}
                </div>
                <div class="col-auto ml-2">
                    <Tooltip>{tooltip()}</Tooltip>
                </div>
            </div>
            <div class="row no-gutters w-100">{render_themes}</div>
        </div>
    }
}

#[tracing::instrument(level = "debug", skip(title, tooltip))]
#[component]
pub fn ExtensionPref<K, H, K1, H1>(#[prop()] title: K, #[prop()] tooltip: K1) -> impl IntoView
where
    K: Fn() -> H + Send + Sync + 'static,
    H: IntoView + Copy + 'static,
    K1: Fn() -> H1 + Send + Sync + 'static,
    H1: IntoView + Copy + 'static,
{
    let extensions = RwSignal::<Vec<ExtensionDetail>>::new(Default::default());
    let fetch_extensions = move || {
        spawn_local(async move {
            let res = get_installed_extensions().await;
            match res {
                Ok(val) => {
                    extensions.set(val);
                }
                Err(e) => {
                    tracing::error!("Failed to get installed extensions {:?}", e);
                }
            }
        })
    };
    fetch_extensions();

    let i18n = use_i18n();

    let extension_path = RwSignal::new(String::new());
    let install_extension = move |_| {
        open_file_browser_single(
            false,
            vec![DialogFilter {
                name: "Moosync extension".into(),
                extensions: vec!["msox".into(), "zip".into()],
            }],
            extension_path,
        );
    };

    Effect::new(move || {
        let extension_path = extension_path.get();
        if extension_path.is_empty() {
            return;
        }

        spawn_local(async move {
            crate::utils::invoke::install_extension(extension_path)
                .await
                .unwrap();
            fetch_extensions()
        });
    });

    let modal_store = expect_context::<RwSignal<ModalStore>>();

    view! {
        <div class="container-fluid mt-4">
            <div class="row no-gutters align-items-center">
                <div class="row no-gutters">
                    <div class="col-auto align-self-center title d-flex preference-title">
                        {title()}
                    </div>
                    <div class="col-auto ml-2">
                        <Tooltip>{tooltip()}</Tooltip>
                    </div>
                </div>
                <div class="col-auto new-directories ml-auto justify-content-center">
                    <div on:click=move |_| {
                        modal_store
                            .update(|m| {
                                m.set_active_modal(Modals::DiscoverExtensions);
                                m.on_modal_close(move || {
                                    tracing::info!("Fetching extensions");
                                    fetch_extensions()
                                });
                            })
                    }>{"Discover"}</div>
                </div>
                <div class="col-auto new-directories ml-4">
                    <div class="add-directories-button" on:click=install_extension>
                        {"Install from file"}
                    </div>
                </div>
            </div>
            <div class="row no-gutters path-prefs-background w-100 mt-2 d-flex">
                <For
                    each=move || extensions.get()
                    key=|e| e.clone()
                    children=move |extension: ExtensionDetail| {
                        view! {
                            <div class="row no-gutters mt-3 item w-100">
                                <div class="col col-md-8 col-lg-9 align-self-center justify-content-start ml-3 no-checkbox-margin">
                                    <div class="item-text text-truncate">
                                        {extension.name.clone()}
                                    </div>
                                </div>
                                <div class="col-auto align-self-center ml-auto">
                                    <div
                                        class="remove-button w-100"
                                        on:click=move |_| {
                                            let package_name = extension.package_name.clone();
                                            spawn_local(async move {
                                                remove_extension(package_name).await.unwrap();
                                                fetch_extensions()
                                            });
                                        }
                                    >
                                        {t!(i18n, settings.paths.remove)}
                                    </div>
                                </div>
                            </div>
                        }
                    }
                />
            </div>
            <div class="row no-gutters w-100 mt-4 d-flex">
                <For
                    each=move || extensions.get()
                    key=|e| e.package_name.clone()
                    children=move |extension: ExtensionDetail| {
                        view! {
                            <div class="container-fluid">
                                <div class="row no-gutters align-items-center">
                                    <div class="row no-gutters">
                                        <div class="col-auto align-self-center title d-flex preference-title">
                                            {extension.name}
                                        </div>

                                    </div>
                                </div>

                                <For
                                    each=move || extension.preferences.clone()
                                    key=|p| p.key.clone()
                                    children=move |preference| {
                                        let title = preference.title;
                                        let tooltip = preference.description;
                                        let key = format!("extensions.{}", preference.key);
                                        match preference._type {
                                            types::preferences::PreferenceTypes::DirectoryGroup => {
                                                view! {
                                                    <PathsPref
                                                        key=key
                                                        title=move || title.clone()
                                                        tooltip=move || tooltip.clone()
                                                        mobile=true
                                                    />
                                                }
                                                    .into_any()
                                            }
                                            types::preferences::PreferenceTypes::EditText => {
                                                view! {
                                                    <InputPref
                                                        key=key
                                                        title=move || title.clone()
                                                        tooltip=move || tooltip.clone()
                                                        show_input=true
                                                        inp_type=serde_wasm_bindgen::to_value(
                                                                &preference.input_type,
                                                            )
                                                            .unwrap()
                                                            .as_string()
                                                            .unwrap_or("text".to_string())
                                                        mobile=true
                                                        is_secure=preference
                                                            .input_type
                                                            .is_some_and(|v| v == InputType::SecureText)
                                                    />
                                                }
                                                    .into_any()
                                            }
                                            types::preferences::PreferenceTypes::FilePicker => {
                                                view! {
                                                    <InputPref
                                                        key=key
                                                        title=move || title.clone()
                                                        tooltip=move || tooltip.clone()
                                                        show_input=false
                                                        inp_type="".to_string()
                                                        mobile=true
                                                    />
                                                }
                                                    .into_any()
                                            }
                                            types::preferences::PreferenceTypes::CheckboxGroup => {
                                                view! {
                                                    <CheckboxPref
                                                        key=key
                                                        title=move || title.clone()
                                                        tooltip=move || tooltip.clone()
                                                        items=preference.items.unwrap_or_default()
                                                        single=preference.single.unwrap_or_default()
                                                        mobile=true
                                                    />
                                                }
                                                    .into_any()
                                            }
                                            types::preferences::PreferenceTypes::Dropdown => {
                                                view! {
                                                    <DropdownPref
                                                        key=key
                                                        title=move || title.clone()
                                                        tooltip=move || tooltip.clone()
                                                        mobile=true
                                                    />
                                                }
                                                    .into_any()
                                            }
                                            types::preferences::PreferenceTypes::ThemeSelector
                                            | types::preferences::PreferenceTypes::Extensions => {
                                                ().into_any()
                                            }
                                            types::preferences::PreferenceTypes::ButtonGroup
                                            | types::preferences::PreferenceTypes::ProgressBar
                                            | types::preferences::PreferenceTypes::TextField
                                            | types::preferences::PreferenceTypes::InfoField => {
                                                ().into_any()
                                            }
                                        }
                                    }
                                />
                            </div>
                        }
                    }
                />

            </div>
        </div>
    }
}

#[tracing::instrument(level = "debug", skip(key, title, tooltip, mobile))]
#[component]
pub fn DropdownPref<K, H, K1, H1>(
    #[prop()] key: String,
    #[prop()] title: K,
    #[prop()] tooltip: K1,
    #[prop()] mobile: bool,
) -> impl IntoView
where
    K: Fn() -> H + Send + Sync + 'static,
    H: IntoView + 'static,
    K1: Fn() -> H1 + Send + Sync + 'static,
    H1: IntoView + 'static,
{
    let ui_store = expect_context::<RwSignal<UiStore>>();
    let is_mobile = create_read_slice(ui_store, |u| u.get_is_mobile()).get();
    if is_mobile && !mobile {
        return ().into_any();
    }

    let should_write = RwSignal::new(false);
    let pref_value = RwSignal::<Vec<CheckboxPreference>>::new(Default::default());
    let pref_key = key;
    load_selective(pref_key.clone(), pref_value.write_only());

    Effect::new(move || {
        let value = pref_value.get();
        if !should_write.get_untracked() {
            untrack(|| should_write.set(true));
            return;
        }

        save_selective(pref_key.clone(), value.clone());
    });

    let selected = create_read_slice(pref_value, |v| {
        v.iter().find(|i| i.enabled).map(|v| v.key.clone())
    });

    view! {
        <div class="container-fluid mt-4">
            <div class="row no-gutters">
                <div class="col-auto align-self-center title d-flex preference-title">
                    {title()}
                </div>
                <div class="col-auto ml-2">
                    <Tooltip>{tooltip()}</Tooltip>
                </div>
            </div>

            <div class="row no-gutters">
                <div class="col-auto">
                    <select
                        class="dropdown-list"
                        on:change=move |ev| {
                            let value = event_target_value(&ev);
                            pref_value
                                .update(|v| {
                                    for item in v {
                                        item.enabled = item.key == value;
                                    }
                                });
                        }
                    >
                        <For
                            each=move || pref_value.get()
                            key=|v| v.key.clone()
                            children=move |val| {
                                let key = val.key.clone();
                                let key1 = val.key.clone();
                                view! {
                                    <option
                                        prop:selected=move || {
                                            if let Some(selected) = selected.get() {
                                                selected == key1.clone()
                                            } else {
                                                false
                                            }
                                        }
                                        value=val.key.clone()
                                    >
                                        {key}
                                    </option>
                                }
                            }
                        />
                    </select>
                </div>
            </div>

        </div>
    }
    .into_any()
}
