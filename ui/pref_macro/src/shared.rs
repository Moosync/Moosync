use quote::quote;

#[cfg(test)]
mod shared_test;

#[derive(Debug)]
pub struct Preference {
    pub id: String,
    pub component: String,
    pub title: String,
    pub subtitle: String,
    pub placeholder: String,
    pub options_radio: Vec<(String, String)>,
    pub options_dropdown: Vec<String>,
}

pub fn parse_yaml(content: &str) -> Vec<Preference> {
    let mut prefs = Vec::new();
    let mut current_pref: Option<Preference> = None;
    let mut current_radio_option: Option<(String, String)> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let indent = line.len() - line.trim_start().len();

        if indent == 2 && trimmed.starts_with("- id:") {
            if let Some(r_opt) = current_radio_option.take() {
                if let Some(ref mut p) = current_pref {
                    p.options_radio.push(r_opt);
                }
            }
            if let Some(p) = current_pref.take() {
                prefs.push(p);
            }
            let val = trimmed["- id:".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
            current_pref = Some(Preference {
                id: val,
                component: String::new(),
                title: String::new(),
                subtitle: String::new(),
                placeholder: String::new(),
                options_radio: Vec::new(),
                options_dropdown: Vec::new(),
            });
            continue;
        }

        if indent == 6 && trimmed.starts_with("- id:") {
            if let Some(r_opt) = current_radio_option.take() {
                if let Some(ref mut p) = current_pref {
                    p.options_radio.push(r_opt);
                }
            }
            let val = trimmed["- id:".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
            current_radio_option = Some((val, String::new()));
            continue;
        }

        if indent == 8 && trimmed.starts_with("label:") {
            let val = trimmed["label:".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
            if let Some(ref mut r_opt) = current_radio_option {
                r_opt.1 = val;
            }
            continue;
        }

        if indent == 6 && trimmed.starts_with("-") {
            let val = trimmed["-".len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
            if let Some(ref mut p) = current_pref {
                p.options_dropdown.push(val);
            }
            continue;
        }

        if let Some(colon_idx) = trimmed.find(':') {
            let key = trimmed[..colon_idx].trim();
            let val = trimmed[colon_idx + 1..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');

            if let Some(ref mut p) = current_pref {
                if key == "component" {
                    p.component = val.to_string();
                } else if key == "title" {
                    p.title = val.to_string();
                } else if key == "subtitle" {
                    p.subtitle = val.to_string();
                } else if key == "placeholder" {
                    p.placeholder = val.to_string();
                }
            }
        }
    }

    if let Some(r_opt) = current_radio_option.take() {
        if let Some(ref mut p) = current_pref {
            p.options_radio.push(r_opt);
        }
    }
    if let Some(p) = current_pref.take() {
        prefs.push(p);
    }

    prefs
}

pub fn generate_expansion(
    content: &str,
    property_name: &syn::Ident,
    handler: &syn::Ident,
) -> proc_macro2::TokenStream {
    let prefs = parse_yaml(content);

    let mut rebuild_blocks = Vec::new();
    let mut change_cases = Vec::new();
    let mut id_match_cases = Vec::new();

    let setter_name = quote::format_ident!("set_{}", property_name);
    let getter_name = quote::format_ident!("get_{}", property_name);

    for p in prefs {
        let id_str = &p.id;
        let component_str = &p.component;
        let title_str = &p.title;
        let subtitle_str = &p.subtitle;
        let placeholder_str = &p.placeholder;

        let radio_options_tokens: Vec<_> = p
            .options_radio
            .iter()
            .map(|(opt_id, opt_label)| {
                quote! {
                    (#opt_id.to_string(), #opt_label.to_string())
                }
            })
            .collect();

        let dropdown_options_tokens: Vec<_> = p
            .options_dropdown
            .iter()
            .map(|o| {
                quote! {
                    #o.to_string()
                }
            })
            .collect();

        let id_ident = quote::format_ident!("{}", id_str);

        let load_value_block = if p.component == "PathSelector" || p.component == "TextArrayInput" {
            quote! {
                let val_list = config.load(preferences::keys::#id_ident).unwrap_or_default();
                let val_string = String::new();
                let val_bool = false;
                let val_number = 0.0f32;
            }
        } else if p.component == "ToggleGroup" {
            quote! {
                let val_list = Vec::new();
                let val_string = String::new();
                let val_bool = config.load(preferences::keys::#id_ident).unwrap_or(false);
                let val_number = 0.0f32;
            }
        } else if p.component == "NumberInputGroup" {
            quote! {
                let val_list = Vec::new();
                let val_int = config.load(preferences::keys::#id_ident).unwrap_or(0);
                let val_string = val_int.to_string();
                let val_bool = false;
                let val_number = val_int as f32;
            }
        } else {
            quote! {
                let val_list = Vec::new();
                let val_string = config.load(preferences::keys::#id_ident).unwrap_or_default();
                let val_bool = false;
                let val_number = 0.0f32;
            }
        };

        rebuild_blocks.push(quote! {
            {
                #load_value_block
                items.push(TempPrefItem {
                    id: #id_str.to_string(),
                    kind: #component_str.to_string(),
                    title: #title_str.to_string(),
                    subtitle: #subtitle_str.to_string(),
                    placeholder: #placeholder_str.to_string(),
                    value_string: val_string,
                    value_bool: val_bool,
                    value_number: val_number,
                    value_list: val_list,
                    options_radio: vec![#(#radio_options_tokens),*],
                    options_dropdown: vec![#(#dropdown_options_tokens),*],
                });
            }
        });

        let save_block = if p.component == "PathSelector" || p.component == "TextArrayInput" {
            quote! {
                let mut current_list = config.load(preferences::keys::#id_ident).unwrap_or_default();
                if !value_list.is_empty() {
                    current_list = value_list.clone();
                } else if !value_string.is_empty() {
                    let val_str = value_string.clone();
                    if current_list.contains(&val_str) {
                        current_list.retain(|x| x != &val_str);
                    } else {
                        current_list.push(val_str);
                    }
                }
                let _ = config.save(preferences::keys::#id_ident, current_list.clone());
                updated_list = Some(current_list);
            }
        } else if p.component == "ToggleGroup" {
            quote! {
                let _ = config.save(preferences::keys::#id_ident, value_bool);
            }
        } else if p.component == "NumberInputGroup" {
            quote! {
                let val: i32 = value_string.parse().unwrap_or(value_number as i32);
                let _ = config.save(preferences::keys::#id_ident, val);
            }
        } else {
            quote! {
                let _ = config.save(preferences::keys::#id_ident, value_string.clone());
            }
        };

        change_cases.push(quote! {
            #id_str => {
                #save_block
            }
        });

        id_match_cases.push(quote! {
            #id_str => true,
        });
    }

    quote! {
        #[derive(Debug, Clone, Default)]
        struct TempPrefItem {
            id: String,
            kind: String,
            title: String,
            subtitle: String,
            placeholder: String,
            value_string: String,
            value_bool: bool,
            value_number: f32,
            value_list: Vec<String>,
            options_radio: Vec<(String, String)>,
            options_dropdown: Vec<String>,
        }

        pub fn init(
            main_window: &crate::MainWindow,
            state_manager: &state_manager::StateManager,
        ) {
            use slint::ComponentHandle;

            let main_window_weak = main_window.as_weak();
            let state_manager = state_manager.clone();

            tokio::spawn(async move {
                let config = state_manager.get_preference_config().await;
                let mut items = Vec::new();
                #(#rebuild_blocks)*

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(main_window) = main_window_weak.upgrade() {
                        let slint_items: Vec<crate::PreferenceItem> = items.into_iter().map(|item| {
                            let radio_items: Vec<crate::RadioItem> = item.options_radio.into_iter().map(|(id, label)| {
                                crate::RadioItem {
                                    id: id.into(),
                                    label: label.into(),
                                }
                            }).collect();
                            let dropdown_items: Vec<slint::SharedString> = item.options_dropdown.into_iter().map(|s| s.into()).collect();
                            let list_items: Vec<slint::SharedString> = item.value_list.into_iter().map(|s| s.into()).collect();

                            crate::PreferenceItem {
                                id: item.id.into(),
                                kind: item.kind.into(),
                                title: item.title.into(),
                                subtitle: item.subtitle.into(),
                                placeholder: item.placeholder.into(),
                                value_string: item.value_string.into(),
                                value_bool: item.value_bool,
                                value_number: item.value_number,
                                value_list: slint::ModelRc::new(slint::VecModel::from(list_items)),
                                options_radio: slint::ModelRc::new(slint::VecModel::from(radio_items)),
                                options_dropdown: slint::ModelRc::new(slint::VecModel::from(dropdown_items)),
                            }
                        }).collect();

                        let prefs_global = main_window.global::<crate::AppPreferences>();
                        prefs_global.#setter_name(slint::ModelRc::new(slint::VecModel::from(slint_items)));
                    }
                });
            });
        }

        impl<'a> crate::settings::PreferenceHandler for #handler<'a> {
            #[tracing::instrument(level = "debug", skip_all)]
            fn init_preferences(&self) {
                init(self.main_window, self.state_manager);
            }

            #[tracing::instrument(level = "debug", skip_all)]
            fn handle_preference_change(
                &self,
                change: &crate::PreferenceChange,
                main_window_weak: &slint::Weak<crate::MainWindow>,
                state_manager: &state_manager::StateManager,
            ) -> bool {
                use slint::Model;

                let id = change.id.to_string();
                let main_window_weak = main_window_weak.clone();

                let matches = match id.as_str() {
                    #(#id_match_cases)*
                    _ => false,
                };

                if !matches {
                    return false;
                }

                let value_string = change.value_string.to_string();
                let value_bool = change.value_bool;
                let value_number = change.value_number;
                let value_list: Vec<String> = change.value_list.iter().map(|s| s.to_string()).collect();
                let state_manager = state_manager.clone();
                tokio::spawn(async move {
                    let mut config = state_manager.get_preference_config_mut().await;
                    let mut updated_list: Option<Vec<String>> = None;
                    match id.as_str() {
                        #(#change_cases)*
                        _ => {}
                    }

                    let _ = slint::invoke_from_event_loop(move || {
                        use slint::{ComponentHandle, Model};

                        if let Some(main_window) = main_window_weak.upgrade() {
                            let prefs_global = main_window.global::<crate::AppPreferences>();
                            let model_rc = prefs_global.#getter_name();
                            if let Some(vec_model) = model_rc.as_any().downcast_ref::<slint::VecModel<crate::PreferenceItem>>() {
                                for idx in 0..vec_model.row_count() {
                                    if let Some(mut item) = vec_model.row_data(idx) {
                                        if item.id == id {
                                            item.value_bool = value_bool;
                                            item.value_string = value_string.clone().into();
                                            item.value_number = value_number;
                                            if let Some(ref list) = updated_list {
                                                let list_items: Vec<slint::SharedString> = list.iter().map(|s| slint::SharedString::from(s.as_str())).collect();
                                                item.value_list = slint::ModelRc::new(slint::VecModel::from(list_items));
                                            } else if !value_list.is_empty() {
                                                let list_items: Vec<slint::SharedString> = value_list.iter().map(|s| slint::SharedString::from(s.as_str())).collect();
                                                item.value_list = slint::ModelRc::new(slint::VecModel::from(list_items));
                                            }
                                            vec_model.set_row_data(idx, item);
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    });
                });

                true
            }
        }
    }
}
