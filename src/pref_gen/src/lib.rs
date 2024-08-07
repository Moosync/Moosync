use std::{fs, path::PathBuf};

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};
use types::ui::preferences::{PreferenceTypes, PreferenceUIData, PreferenceUIFile};

fn get_path(path_lit: String) -> proc_macro2::TokenStream {
    let path_parts: Vec<syn::Ident> = path_lit
        .split('.')
        .map(|part| syn::Ident::new(part, proc_macro2::Span::call_site()))
        .collect();

    let mut access = quote! {};
    for part in path_parts {
        if access.is_empty() {
            access = quote! { #part };
        } else {
            access = quote! { #access.#part };
        }
    }

    access
}

#[proc_macro]
pub fn generate_components(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let relative_path = input.value();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

    // Construct the full path to the YAML file
    let full_path = PathBuf::from(manifest_dir).join(relative_path);

    let yaml_content = fs::read_to_string(full_path.clone()).expect("Unable to read file");
    let config: PreferenceUIFile =
        serde_yaml::from_str(&yaml_content).expect("Invalid YAML format");

    let components = generate_component(&config);
    TokenStream::from(components)
}

fn generate_component(config: &PreferenceUIFile) -> proc_macro2::TokenStream {
    let mut ret = vec![];
    let mut tabs = vec![];
    let mut routes = vec![];

    for page in &config.page {
        let name = syn::Ident::new(
            format!("Preference{}Page", page.path.clone()).as_str(),
            proc_macro2::Span::call_site(),
        );

        let page_title = get_path(page.title.clone());
        let page_icon = page.icon.clone();
        let page_path = page.path.clone();
        let page_full_path = format!("/prefs/{}", page_path);

        tabs.push(quote! {
            Tab::new(i18n.get_keys().#page_title, #page_icon, #page_full_path),
        });

        routes.push(quote! {
           <Route path=#page_path view=#name  />
        });

        let children = generate_children(&page.data);

        let mut fns = vec![];
        let mut components = vec![];
        for (fn_name, stream) in children {
            fns.push(stream);
            components.push(quote! { <#fn_name /> });
        }

        let stream = quote! {
            #(#fns)*

            #[component]
            pub fn #name() -> impl IntoView {
                view! {
                    <div class="prefs-container">
                        #(#components)*
                    </div>
                }
            }
        };

        ret.push(stream);
    }

    quote! {
        use crate::console_log;
        use crate::icons::{folder_icon::FolderIcon, theme_view_icon::ThemeViewIcon};
        use crate::{
            i18n::use_i18n,
            icons::tooltip::Tooltip,
            utils::prefs::{load_selective, load_selective_async, open_file_browser, open_file_browser_single, save_selective, save_selective_number},
            utils::common::invoke,
        };
        use leptos::{
            component, create_effect, create_rw_signal, event_target_checked, event_target_value, view,
            CollectView, For, IntoView, SignalGet, SignalSet, SignalUpdate, RwSignal, SignalSetUntracked
        };
        use leptos_i18n::t;
        use leptos_use::use_debounce_fn_with_arg;
        use types::preferences::CheckboxPreference;
        use crate::components::sidebar::{Sidebar, Tab};
        use leptos_router::{Outlet, Redirect, Route, Router, Routes};
        use wasm_bindgen::JsValue;
        use wasm_bindgen_futures::spawn_local;
        use types::themes::ThemeDetails;
        use std::collections::HashMap;


        #(#ret)*

        #[component]
        pub fn PrefApp() -> impl IntoView {
            let i18n = use_i18n();
            let mut tabs = vec![
                #(#tabs)*
            ];
            view! {
                <div>
                    <Sidebar tabs=tabs />
                    <div class="main-container">
                        <Outlet />
                    </div>
                </div>
            }
        }

        #[component]
        pub fn RedirectPrefs() -> impl IntoView {
            view! { <Redirect path="/prefs/paths" /> }
        }

        #[component(transparent)]
        pub fn SettingRoutes() -> impl IntoView {
            view! {
                <Route path="/prefs" view=PrefApp >
                    #(#routes)*
                    <Route path="" view=RedirectPrefs />
                </Route>
            }
        }
    }
}

fn generate_children(data: &[PreferenceUIData]) -> Vec<(syn::Ident, proc_macro2::TokenStream)> {
    let mut ret = vec![];

    for item in data {
        let stream = match item._type {
            types::ui::preferences::PreferenceTypes::Paths => generate_paths(item),
            types::ui::preferences::PreferenceTypes::Text
            | types::ui::preferences::PreferenceTypes::Number
            | types::ui::preferences::PreferenceTypes::FilePicker => generate_input(item),
            types::ui::preferences::PreferenceTypes::Checkbox => generate_checkbox(item),
            types::ui::preferences::PreferenceTypes::ThemeSelector => generate_themes(item),
        };
        ret.push(stream);
    }

    ret
}

fn generate_checkbox(data: &PreferenceUIData) -> (syn::Ident, proc_macro2::TokenStream) {
    let key = data.key.clone();

    let name = get_path(data.name.clone());

    let tooltip = get_path(data.tooltip.clone());

    let fn_name = syn::Ident::new(
        format!("Checkbox{}Pref", data.key)
            .replace(".", "")
            .as_str(),
        proc_macro2::Span::call_site(),
    );

    let single = data.single.unwrap_or(false);

    let mut checkboxes = vec![];
    for items in data.items.clone().unwrap() {
        let item_key = items.key.clone();
        let item_name = get_path(items.name.clone());
        let checkbox_key = format!("checkbox-{}-{}", key, item_key);

        let stream = quote! {
            <div class="row no-gutters item w-100">
                <div class="col-auto align-self-center">
                    <div class="custom-control custom-checkbox">
                        <input
                            type="checkbox"
                            class="custom-control-input"
                            prop:checked=move || {
                                pref_value
                                    .get()
                                    .iter()
                                    .find(|val| val.key == #item_key)
                                    .map(|item| item.enabled)
                                    .unwrap_or(false)
                            }
                            id=#checkbox_key
                            on:change=move |ev| {
                                let enabled = event_target_checked(&ev);
                                pref_value
                                    .update(|val| {
                                        let found = val
                                            .iter_mut()
                                            .find(|val| val.key == #item_key);
                                        if let Some(item) = found {
                                            item.enabled = enabled;
                                        } else {
                                            val.push(CheckboxPreference {
                                                key: #item_key.to_string(),
                                                enabled,
                                            });
                                        }

                                        if enabled {
                                            last_enabled.set(#item_key.to_string());
                                        }
                                    });
                            }
                        />
                        <label
                            for=#checkbox_key
                            class="custom-control-label"
                        ></label>
                    </div>
                </div>

                <div class="col-md-8 col-lg-9 col align-self-center ml-3 justify-content-start">
                    <div class="item-text text-truncate">{t!(i18n, #item_name)}</div>
                </div>
            </div>
        };

        checkboxes.push(stream);
    }

    let stream = quote! {
        #[component()]
        pub fn #fn_name() -> impl IntoView {
            let should_write = create_rw_signal(false);
            let pref_value = create_rw_signal::<Vec<CheckboxPreference>>(Default::default());
            let pref_key = #key.to_string();
            load_selective(pref_key.clone(), pref_value.write_only());

            let last_enabled = create_rw_signal(String::new());

            create_effect(move |_| {
                let mut value = pref_value.get();
                if !should_write.get() {
                    should_write.set(true);
                    return;
                }

                if #single {
                    let last_enabled = last_enabled.get();
                    // let mut value_c = value.clone();
                    for items in value.iter_mut() {
                        items.enabled = items.key == last_enabled;
                    }
                }
                save_selective(pref_key.clone(), value.clone());
                // pref_value.set_untracked(value);

            });
            let i18n = use_i18n();
            view! {
                <div class="container-fluid mt-4">

                    <div class="row no-gutters">
                        <div class="col-auto align-self-center title d-flex preference-title">
                            {t!(i18n, #name)}
                        </div>
                        <div class="col-auto ml-2">
                            <Tooltip> {t!(i18n, #tooltip)} </Tooltip>
                        </div>
                    </div>

                    #(#checkboxes)*
                </div>
            }
        }
    };

    (fn_name, stream)
}

fn generate_input(data: &PreferenceUIData) -> (syn::Ident, proc_macro2::TokenStream) {
    let key = data.key.clone();

    let name = get_path(data.name.clone());

    let tooltip = get_path(data.tooltip.clone());

    let (show_input, inp_type, is_number) = match data._type {
        PreferenceTypes::FilePicker => (false, "", false),
        PreferenceTypes::Text => (true, "text", false),
        PreferenceTypes::Number => (true, "number", true),
        // Below case should never happen
        _ => (true, "", false),
    };

    let fn_name = syn::Ident::new(
        format!("Input{}Pref", data.key).replace(".", "").as_str(),
        proc_macro2::Span::call_site(),
    );

    let stream = quote! {
        #[component()]
        pub fn #fn_name() -> impl IntoView {
            let (show_input, inp_type) = (#show_input, #inp_type);

            let should_write = create_rw_signal(false);
            let pref_value = create_rw_signal(Default::default());
            let pref_key = #key.to_string();
            load_selective(pref_key.clone(), pref_value.write_only());

            create_effect(move |_| {
                let value = pref_value.get();
                if !should_write.get() {
                    should_write.set(true);
                    return;
                }

                if #is_number {
                    save_selective_number(pref_key.clone(), value);
                } else {
                    save_selective(pref_key.clone(), value);
                }

            });

            let debounced_update = use_debounce_fn_with_arg(
                move |event: web_sys::Event| {
                    let value = event_target_value(&event);
                    if #is_number && value.parse::<f64>().is_err() {
                        console_log!("Invalid number");
                        return;
                    }
                    pref_value.set(value);
                },
                500.0,
            );

            let i18n = use_i18n();

            view! {
                <div class="container-fluid  mt-4">
                    <div class="row no-gutters">
                        <div class="col-auto align-self-center title d-flex preference-title">
                            {t!(i18n, #name)}
                        </div>
                        <div class="col-auto ml-2">
                            <Tooltip> {t!(i18n, #tooltip)} </Tooltip>
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
                                .into_view()
                        } else {
                            view! {}.into_view()
                        }}
                        <div class="col-auto ml-3 align-self-center flex-grow-1 justify-content-start">

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
                                    .into_view()
                            } else {
                                view! {
                                    <div class="item-text text-truncate file-picker-text">
                                        {move || pref_value.get()}
                                    </div>
                                }
                                    .into_view()
                            }}
                        </div>
                    </div>

                </div>
            }
        }
    };

    (fn_name, stream)
}

fn generate_paths(data: &PreferenceUIData) -> (syn::Ident, proc_macro2::TokenStream) {
    let key = data.key.clone();

    let name = get_path(data.name.clone());

    let tooltip = get_path(data.tooltip.clone());

    let fn_name = syn::Ident::new(
        format!("Paths{}Pref", data.key).replace(".", "").as_str(),
        proc_macro2::Span::call_site(),
    );
    let stream = quote! {
        #[component()]
        pub fn #fn_name() -> impl IntoView {
            let should_write = create_rw_signal(false);
            let paths: RwSignal<Vec<String>> = create_rw_signal(vec![]);
            load_selective(#key.to_string(), paths.write_only());

            let selected_paths = create_rw_signal(vec![]);

            create_effect(move |_| {
                let new_paths = selected_paths.get();
                if !new_paths.is_empty() {
                    paths.update(|paths| paths.extend(new_paths.iter().cloned()));
                }
            });

            create_effect(move |_| {
                if !should_write.get() {
                    should_write.set(true);
                    return;
                }
                let value = paths.get();
                save_selective(#key.to_string(), value);
            });

            let i18n = use_i18n();

            view! {
                <div class="container-fluid mt-4">
                    <div class="row no-gutters align-items-center">

                        <div class="row no-gutters">
                            <div class="col-auto align-self-center title d-flex preference-title">
                                {t!(i18n, #name)}
                            </div>
                            <div class="col-auto ml-2">
                                <Tooltip> {t!(i18n, #tooltip)} </Tooltip>
                            </div>
                        </div>

                        <div class="col-auto new-directories ml-auto justify-content-center">
                            <div on:click=move |_| {
                                save_selective(#key.to_string(), paths.get())
                                } >{"Refresh"}</div>
                        </div>

                        <div class="col-auto new-directories ml-4">
                            <div
                                class="add-directories-button"
                                on:click=move |_| open_file_browser(true, true, vec![], selected_paths)
                            >
                                {t!(i18n, settings.paths.addFolder)}
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
            }
        }
    };

    (fn_name, stream)
}

fn generate_themes(data: &PreferenceUIData) -> (syn::Ident, proc_macro2::TokenStream) {
    let key = data.key.clone();

    let name = get_path(data.name.clone());

    let tooltip = get_path(data.tooltip.clone());

    let fn_name = syn::Ident::new(
        format!("Themes{}Pref", data.key).replace(".", "").as_str(),
        proc_macro2::Span::call_site(),
    );

    let stream = quote! {
        #[component]
        pub fn #fn_name() -> impl IntoView {
            let all_themes: RwSignal<HashMap<String, ThemeDetails>> = create_rw_signal(Default::default());
            spawn_local(async move {
                let themes = invoke("load_all_themes", JsValue::undefined())
                    .await
                    .unwrap();
                all_themes.set(serde_wasm_bindgen::from_value(themes).unwrap());
            });

            let active_themes = create_rw_signal(vec![]);

            let active_theme_id = create_rw_signal(String::new());
            load_selective(#key.into(), active_theme_id);

            let render_themes = move || {
                let mut views = vec![];
                let active_theme_id = active_theme_id.get();
                for (key, theme) in all_themes.get() {
                    let signal = create_rw_signal(key == active_theme_id);
                    active_themes.update(|at| at.push(signal));
                    views.push(view! {
                        <div class="col-xl-3 col-5 p-2">
                            <div
                                class="theme-component-container"
                                on:click=move |_| {
                                    active_themes
                                        .update(|at| {
                                            for s in at.iter() {
                                                s.set(false);
                                            }
                                            signal.set(true);
                                        });
                                    let theme_id = key.clone();
                                    console_log!("Setting active theme: {}", theme_id);
                                    save_selective(#key.into(), theme_id);
                                }
                            >
                                <ThemeViewIcon active=signal.read_only() theme=theme.clone() />
                                <div class="theme-title">{theme.name}</div>
                                <div class="theme-author">{theme.author}</div>
                            </div>
                        </div>
                    });
                }
                views.collect_view()
            };

            let i18n = use_i18n();
            view! {
                <div class="container-fluid">
                    <div class="row no-gutters">
                        <div class="col-auto align-self-center title d-flex preference-title">
                            {t!(i18n, #name)}
                        </div>
                        <div class="col-auto ml-2">
                            <Tooltip>{t!(i18n, #tooltip)}</Tooltip>
                        </div>
                    </div>

                    <div class="row no-gutters w-100">{render_themes}</div>

                </div>
            }
        }
    };

    (fn_name, stream)
}
