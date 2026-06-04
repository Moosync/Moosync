use std::fs;

use quote::quote;

#[derive(Debug)]
struct Preference {
    id: String,
    component: String,
    title: String,
    subtitle: String,
    placeholder: String,
    options_radio: Vec<(String, String)>,
    options_dropdown: Vec<String>,
}

fn parse_yaml(content: &str) -> Vec<Preference> {
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

fn generate_expansion(content: &str, property_name: &str) -> String {
    let prefs = parse_yaml(content);
    let setter_name = quote::format_ident!("set_{}", property_name);

    let mut rebuild_blocks = Vec::new();
    let mut id_match_cases = Vec::new();
    let mut change_cases = Vec::new();

    for p in &prefs {
        let id_str = &p.id;
        let component_str = &p.component;
        let title_str = &p.title;
        let subtitle_str = &p.subtitle;
        let placeholder_str = &p.placeholder;

        id_match_cases.push(quote! {
            #id_str => true,
        });

        let radio_options_tokens = p.options_radio.iter().map(|(oid, olabel)| {
            quote! {
                (#oid.to_string(), #olabel.to_string())
            }
        });

        let dropdown_options_tokens = p.options_dropdown.iter().map(|o| {
            quote! {
                #o.to_string()
            }
        });

        let load_value_block = if p.component == "PathSelector" {
            quote! {
                let val_list = config.load_selective::<Vec<String>>(#id_str.to_string()).unwrap_or_default();
                let val_string = String::new();
                let val_bool = false;
                let val_number = 0.0f32;
            }
        } else if p.component == "ToggleGroup" {
            quote! {
                let val_list = Vec::new();
                let val_string = String::new();
                let val_bool = config.load_selective::<bool>(#id_str.to_string()).unwrap_or(false);
                let val_number = 0.0f32;
            }
        } else if p.component == "NumberInputGroup" {
            quote! {
                let val_list = Vec::new();
                let val_int = config.load_selective::<i32>(#id_str.to_string()).unwrap_or(0);
                let val_string = val_int.to_string();
                let val_bool = false;
                let val_number = val_int as f32;
            }
        } else {
            quote! {
                let val_list = Vec::new();
                let val_string = config.load_selective::<String>(#id_str.to_string()).unwrap_or_default();
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

        let save_block = if p.component == "PathSelector" {
            quote! {
                let mut current_list = config.load_selective::<Vec<String>>(#id_str.to_string()).unwrap_or_default();
                let val_str = change.value_string.to_string();
                if current_list.contains(&val_str) {
                    current_list.retain(|x| x != &val_str);
                } else {
                    let next_idx = current_list.len() + 1;
                    let folder_name = format!("{}_{}", val_str, next_idx);
                    current_list.push(folder_name);
                }
                let _ = config.save_selective(#id_str.to_string(), Some(current_list));
            }
        } else if p.component == "ToggleGroup" {
            quote! {
                let _ = config.save_selective(#id_str.to_string(), Some(change.value_bool));
            }
        } else if p.component == "NumberInputGroup" {
            quote! {
                let val: i32 = change.value_string.parse().unwrap_or(change.value_number as i32);
                let _ = config.save_selective(#id_str.to_string(), Some(val));
            }
        } else {
            quote! {
                let _ = config.save_selective(#id_str.to_string(), Some(change.value_string.to_string()));
            }
        };

        change_cases.push(quote! {
            #id_str => {
                #save_block
            }
        });
    }

    let expanded = quote! {
        use slint::ComponentHandle;

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

        fn rebuild_items(config: &preferences::preferences::PreferenceConfig) -> Vec<TempPrefItem> {
            let mut items = Vec::new();
            #(#rebuild_blocks)*
            items
        }

        pub fn init(main_window: &crate::MainWindow, state_manager: &state_manager::StateManager) {
            let main_window_weak = main_window.as_weak();
            let state_manager = state_manager.clone();
            tokio::spawn(async move {
                let config = state_manager.get_preference_config().await;
                let temp_items = rebuild_items(&*config.inner);

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(main_window) = main_window_weak.upgrade() {
                        let slint_items: Vec<crate::PreferenceItem> = temp_items.into_iter().map(|item| {
                            crate::PreferenceItem {
                                id: item.id.into(),
                                kind: item.kind.into(),
                                title: item.title.into(),
                                subtitle: item.subtitle.into(),
                                placeholder: item.placeholder.into(),
                                value_string: item.value_string.into(),
                                value_bool: item.value_bool,
                                value_number: item.value_number,
                                value_list: slint::ModelRc::new(slint::VecModel::from(
                                    item.value_list.into_iter().map(slint::SharedString::from).collect::<Vec<_>>()
                                )),
                                options_radio: slint::ModelRc::new(slint::VecModel::from(
                                    item.options_radio.into_iter().map(|(oid, olabel)| crate::RadioItem {
                                        id: oid.into(),
                                        label: olabel.into(),
                                    }).collect::<Vec<_>>()
                                )),
                                options_dropdown: slint::ModelRc::new(slint::VecModel::from(
                                    item.options_dropdown.into_iter().map(slint::SharedString::from).collect::<Vec<_>>()
                                )),
                            }
                        }).collect();

                        let prefs_global = main_window.global::<crate::AppPreferences>();
                        prefs_global.#setter_name(slint::ModelRc::new(slint::VecModel::from(slint_items)));
                    }
                });
            });
        }

        pub fn handle_change(
            change: &crate::PreferenceChange,
            main_window_weak: &slint::Weak<crate::MainWindow>,
            state_manager: &state_manager::StateManager,
        ) -> bool {
            let id = change.id.to_string();
            let main_window_weak = main_window_weak.clone();

            let matches = match id.as_str() {
                #(#id_match_cases)*
                _ => false,
            };

            if !matches {
                return false;
            }

            let change = change.clone();
            let state_manager = state_manager.clone();
            tokio::spawn(async move {
                let mut config = state_manager.get_preference_config_mut().await;
                match id.as_str() {
                    #(#change_cases)*
                    _ => {}
                }

                let temp_items = rebuild_items(&*config.inner);

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(main_window) = main_window_weak.upgrade() {
                        let slint_items: Vec<crate::PreferenceItem> = temp_items.into_iter().map(|item| {
                            crate::PreferenceItem {
                                id: item.id.into(),
                                kind: item.kind.into(),
                                title: item.title.into(),
                                subtitle: item.subtitle.into(),
                                placeholder: item.placeholder.into(),
                                value_string: item.value_string.into(),
                                value_bool: item.value_bool,
                                value_number: item.value_number,
                                value_list: slint::ModelRc::new(slint::VecModel::from(
                                    item.value_list.into_iter().map(slint::SharedString::from).collect::<Vec<_>>()
                                )),
                                options_radio: slint::ModelRc::new(slint::VecModel::from(
                                    item.options_radio.into_iter().map(|(oid, olabel)| crate::RadioItem {
                                        id: oid.into(),
                                        label: olabel.into(),
                                    }).collect::<Vec<_>>()
                                )),
                                options_dropdown: slint::ModelRc::new(slint::VecModel::from(
                                    item.options_dropdown.into_iter().map(slint::SharedString::from).collect::<Vec<_>>()
                                )),
                            }
                        }).collect();

                        let prefs_global = main_window.global::<crate::AppPreferences>();
                        prefs_global.#setter_name(slint::ModelRc::new(slint::VecModel::from(slint_items)));
                    }
                });
            });

            true
        }
    };

    expanded.to_string()
}

fn main() {
    let workspace = std::env::var("BUILD_WORKSPACE_DIRECTORY").unwrap_or_else(|_| ".".to_string());
    let workspace_path = std::path::PathBuf::from(workspace);

    let scratch_dir = workspace_path.join("scratch");
    let _ = fs::create_dir_all(&scratch_dir);

    // Process paths
    let paths_yaml_path = workspace_path.join("ui/slint/src/settings/paths_prefs.yaml");
    let paths_content = fs::read_to_string(&paths_yaml_path)
        .unwrap_or_else(|e| panic!("Could not read {:?}: {}", paths_yaml_path, e));
    let paths_expanded = generate_expansion(&paths_content, "paths_items");
    let paths_expanded_path = scratch_dir.join("paths_items_expanded.rs");
    fs::write(&paths_expanded_path, paths_expanded).unwrap();

    // Process system
    let system_yaml_path = workspace_path.join("ui/slint/src/settings/system_prefs.yaml");
    let system_content = fs::read_to_string(&system_yaml_path)
        .unwrap_or_else(|e| panic!("Could not read {:?}: {}", system_yaml_path, e));
    let system_expanded = generate_expansion(&system_content, "system_items");
    let system_expanded_path = scratch_dir.join("system_items_expanded.rs");
    fs::write(&system_expanded_path, system_expanded).unwrap();

    println!(
        "Expanded code successfully written to scratch/paths_items_expanded.rs and scratch/system_items_expanded.rs"
    );
}
