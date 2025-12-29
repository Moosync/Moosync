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

use std::{fs, path::PathBuf};

use quote::quote;
use types::preferences::{InputType, PreferenceTypes, PreferenceUIData, PreferenceUIFile};

#[tracing::instrument(level = "debug", skip(path_lit))]
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

#[tracing::instrument(level = "debug", skip(relative_path))]
pub fn generate_components(relative_path: &str) {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

    // Construct the full path to the YAML file
    let full_path = PathBuf::from(manifest_dir).join(relative_path);

    let yaml_content = fs::read_to_string(full_path.clone()).expect("Unable to read file");
    let config: PreferenceUIFile =
        serde_yaml::from_str(&yaml_content).expect("Invalid YAML format");

    let components = generate_component(&config);

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR environment variable not set");
    let dest_path = PathBuf::from(out_dir).join("prefs");
    if !dest_path.exists() {
        fs::create_dir_all(&dest_path).expect("Unable to create output directory");
    }
    let dest_path = dest_path.join("mod.rs");
    fs::write(dest_path, components.to_string()).expect("Unable to write file");
}

#[tracing::instrument(level = "debug", skip(config))]
fn generate_component(config: &PreferenceUIFile) -> proc_macro2::TokenStream {
    let mut ret = vec![];
    let mut tabs = vec![];
    let mut routes = vec![];

    tabs.push(quote! {
        Tab::new(move || "Home", "Home", "/main/allsongs"),
    });
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
            Tab::new(move || t_string!(use_i18n(), #page_title), #page_icon, #page_full_path),
        });

        routes.push(quote! {
           <Route path=path!(#page_path) view=#name  />
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

            #[allow(unused_variables)]
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
        use crate::components::{
            prefs::components::{CheckboxPref, ExtensionPref, InputPref, PathsPref, ThemesPref, DropdownPref},
            sidebar::{Sidebar, Tab},
        };
        use crate::i18n::*;
        use leptos::prelude::*;
        use leptos_i18n::t_string;
        use leptos_router::{
            components::{Outlet, ParentRoute, Redirect, Route},
            path, MatchNestedRoutes,
        };
        use types::preferences::CheckboxItems;
        use crate::store::ui_store::UiStore;

        #(#ret)*

        #[component]
        pub fn PrefApp() -> impl IntoView {
            let ui_store = expect_context::<RwSignal<UiStore>>();
            let is_mobile = create_read_slice(ui_store, |u| u.get_is_mobile()).get();

            let tabs = vec![
                #(#tabs)*
            ];
            view! {
                <div>
                    <Sidebar tabs=tabs />
                        <div class="main-container" class:main-container-mobile=is_mobile>
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
        pub fn SettingRoutes() -> impl MatchNestedRoutes + Clone {
            view! {
                <ParentRoute path=path!("/prefs") view=PrefApp >
                    #(#routes)*
                    <Route path=path!("") view=RedirectPrefs />
                </ParentRoute>
            }.into_inner()
        }
    }
}

#[tracing::instrument(level = "debug", skip(data))]
fn generate_children(data: &[PreferenceUIData]) -> Vec<(syn::Ident, proc_macro2::TokenStream)> {
    let mut ret = vec![];

    for item in data {
        let stream = match item._type {
            types::preferences::PreferenceTypes::DirectoryGroup => generate_paths(item),
            types::preferences::PreferenceTypes::EditText
            | types::preferences::PreferenceTypes::FilePicker => generate_input(item),
            types::preferences::PreferenceTypes::CheckboxGroup => generate_checkbox(item),
            types::preferences::PreferenceTypes::ThemeSelector => generate_themes(item),
            types::preferences::PreferenceTypes::Extensions => generate_extensions(item),
            types::preferences::PreferenceTypes::Dropdown => generate_dropdowns(item),
            types::preferences::PreferenceTypes::ButtonGroup
            | types::preferences::PreferenceTypes::InfoField
            | types::preferences::PreferenceTypes::ProgressBar
            | types::preferences::PreferenceTypes::TextField => continue,
        };
        ret.push(stream);
    }

    ret
}

#[tracing::instrument(level = "debug", skip(data))]
fn generate_checkbox(data: &PreferenceUIData) -> (syn::Ident, proc_macro2::TokenStream) {
    let key = data.key.clone();
    let mobile = data.mobile.unwrap_or(true);

    let name = get_path(data.title.clone());

    let tooltip = get_path(data.description.clone());

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
        let item_name = get_path(items.title.clone());

        let stream = quote! {
            CheckboxItems {
                title: t!(i18n, #item_name)().to_html(),
                key: #item_key.to_string(),
            },
        };

        checkboxes.push(stream);
    }

    let stream = quote! {
        #[component()]
        pub fn #fn_name() -> impl IntoView {
            let i18n = use_i18n();
            let checkbox_items = vec![
                #(#checkboxes)*
            ];



            view! {

                <CheckboxPref
                    mobile=#mobile
                    key=#key.to_string()
                    title=t!(i18n, #name)
                    tooltip=t!(i18n, #tooltip)
                    items=checkbox_items
                    single=#single
                />

            }
        }
    };

    (fn_name, stream)
}

#[tracing::instrument(level = "debug", skip(data))]
fn generate_input(data: &PreferenceUIData) -> (syn::Ident, proc_macro2::TokenStream) {
    let key = data.key.clone();
    let mobile = data.mobile.unwrap_or(true);

    let name = get_path(data.title.clone());

    let tooltip = get_path(data.description.clone());

    let (show_input, inp_type) = match data._type {
        PreferenceTypes::FilePicker => (false, ""),
        PreferenceTypes::EditText => match data.input_type.clone().unwrap() {
            InputType::SecureText => (true, "password"),
            InputType::Text => (true, "text"),
            InputType::Number => (true, "number"),
        },
        // Below case should never happen
        _ => (true, ""),
    };

    let fn_name = syn::Ident::new(
        format!("Input{}Pref", data.key).replace(".", "").as_str(),
        proc_macro2::Span::call_site(),
    );

    let is_secure = data
        .input_type
        .clone()
        .is_some_and(|v| v == InputType::SecureText);

    let stream = quote! {
        #[component()]
        pub fn #fn_name() -> impl IntoView {
            let i18n = use_i18n();

            view! {
                <InputPref key=#key.to_string() title=t!(i18n, #name) tooltip=t!(i18n, #tooltip) show_input=#show_input inp_type=#inp_type.to_string() mobile=#mobile is_secure=#is_secure />
            }
        }
    };

    (fn_name, stream)
}

#[tracing::instrument(level = "debug", skip(data))]
fn generate_paths(data: &PreferenceUIData) -> (syn::Ident, proc_macro2::TokenStream) {
    let key = data.key.clone();
    let mobile = data.mobile.unwrap_or(true);

    let name = get_path(data.title.clone());

    let tooltip = get_path(data.description.clone());

    let fn_name = syn::Ident::new(
        format!("Paths{}Pref", data.key).replace(".", "").as_str(),
        proc_macro2::Span::call_site(),
    );
    let stream = quote! {
        #[component()]
        pub fn #fn_name() -> impl IntoView {
            let i18n = use_i18n();

            view! {
                <PathsPref key=#key.to_string() title=t!(i18n, #name) tooltip=t!(i18n, #tooltip) mobile=#mobile />
            }
        }
    };

    (fn_name, stream)
}

#[tracing::instrument(level = "debug", skip(data))]
fn generate_themes(data: &PreferenceUIData) -> (syn::Ident, proc_macro2::TokenStream) {
    let key = data.key.clone();

    let name = get_path(data.title.clone());

    let tooltip = get_path(data.description.clone());

    let fn_name = syn::Ident::new(
        format!("Themes{}Pref", data.key).replace(".", "").as_str(),
        proc_macro2::Span::call_site(),
    );

    let stream = quote! {
        #[component]
        pub fn #fn_name() -> impl IntoView {
            let i18n = use_i18n();

            view! {
                <ThemesPref key=#key.to_string() title=t!(i18n, #name) tooltip=t!(i18n, #tooltip) />
            }
        }
    };

    (fn_name, stream)
}

#[tracing::instrument(level = "debug", skip(data))]
fn generate_extensions(data: &PreferenceUIData) -> (syn::Ident, proc_macro2::TokenStream) {
    let name = get_path(data.title.clone());

    let tooltip = get_path(data.description.clone());

    let fn_name = syn::Ident::new(
        format!("Extensions{}Pref", data.key)
            .replace(".", "")
            .as_str(),
        proc_macro2::Span::call_site(),
    );

    let stream = quote! {
        #[component]
        pub fn #fn_name() -> impl IntoView {
            let i18n = use_i18n();

            view !{
                <ExtensionPref title=t!(i18n, #name) tooltip=t!(i18n, #tooltip) />
            }
        }
    };

    (fn_name, stream)
}

#[tracing::instrument(level = "debug", skip(data))]
fn generate_dropdowns(data: &PreferenceUIData) -> (syn::Ident, proc_macro2::TokenStream) {
    let name = get_path(data.title.clone());
    let key = data.key.clone();
    let mobile = data.mobile.unwrap_or(true);

    let tooltip = get_path(data.description.clone());

    let fn_name = syn::Ident::new(
        format!("Extensions{}Pref", data.key)
            .replace(".", "")
            .as_str(),
        proc_macro2::Span::call_site(),
    );

    let stream = quote! {
        #[component]
        pub fn #fn_name() -> impl IntoView {
            let i18n = use_i18n();


            view !{
                <DropdownPref title=t!(i18n, #name) tooltip=t!(i18n, #tooltip) key=#key.to_string() mobile=#mobile />
            }
        }
    };

    (fn_name, stream)
}
