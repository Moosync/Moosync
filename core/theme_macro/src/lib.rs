extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};

#[proc_macro]
pub fn generate_theme_impl(input: TokenStream) -> TokenStream {
    let path_str = syn::parse_macro_input!(input as syn::LitStr).value();
    let content = match read_file(&path_str) {
        Ok(c) => c,
        Err(e) => panic!("Failed to read slint constants file '{}': {:?}", path_str, e),
    };
    let properties = parse_slint_theme(&content);

    let mut get_arms = Vec::new();
    let mut set_arms = Vec::new();
    let mut key_inserts = Vec::new();
    let mut default_setters = Vec::new();

    // The 8 proto fields of ThemeItem:
    // primary, secondary, tertiary, text_primary, text_secondary, text_inverse, accent, divider
    for (name, _prop_type, default_val) in properties {
        let name_str = name.as_str();
        
        // Add to keys
        key_inserts.push(quote! {
            keys.insert(#name_str.to_string());
        });

        // Add to get_default_theme_item
        default_setters.push(quote! {
            item.set_constant(#name_str, #default_val.into());
        });

        // Setup proto fields or constants map
        match name_str {
            "primary" => {
                get_arms.push(quote! {
                    "primary" => if !self.primary.is_empty() { Some(self.primary.clone()) } else { self.constants.get(key).cloned() },
                });
                set_arms.push(quote! {
                    "primary" => {
                        self.primary = value;
                        self.constants.remove("primary");
                    }
                });
            }
            "secondary" => {
                get_arms.push(quote! {
                    "secondary" => if !self.secondary.is_empty() { Some(self.secondary.clone()) } else { self.constants.get(key).cloned() },
                });
                set_arms.push(quote! {
                    "secondary" => {
                        self.secondary = value;
                        self.constants.remove("secondary");
                    }
                });
            }
            "tertiary" => {
                get_arms.push(quote! {
                    "tertiary" => if !self.tertiary.is_empty() { Some(self.tertiary.clone()) } else { self.constants.get(key).cloned() },
                });
                set_arms.push(quote! {
                    "tertiary" => {
                        self.tertiary = value;
                        self.constants.remove("tertiary");
                    }
                });
            }
            "textPrimary" => {
                get_arms.push(quote! {
                    "textPrimary" => if !self.text_primary.is_empty() { Some(self.text_primary.clone()) } else { self.constants.get(key).cloned() },
                });
                set_arms.push(quote! {
                    "textPrimary" => {
                        self.text_primary = value;
                        self.constants.remove("textPrimary");
                        self.constants.remove("text_primary");
                    }
                });
            }
            "textSecondary" => {
                get_arms.push(quote! {
                    "textSecondary" => if !self.text_secondary.is_empty() { Some(self.text_secondary.clone()) } else { self.constants.get(key).cloned() },
                });
                set_arms.push(quote! {
                    "textSecondary" => {
                        self.text_secondary = value;
                        self.constants.remove("textSecondary");
                        self.constants.remove("text_secondary");
                    }
                });
            }
            "textInverse" => {
                get_arms.push(quote! {
                    "textInverse" => if !self.text_inverse.is_empty() { Some(self.text_inverse.clone()) } else { self.constants.get(key).cloned() },
                });
                set_arms.push(quote! {
                    "textInverse" => {
                        self.text_inverse = value;
                        self.constants.remove("textInverse");
                        self.constants.remove("text_inverse");
                    }
                });
            }
            "accent" => {
                get_arms.push(quote! {
                    "accent" => if !self.accent.is_empty() { Some(self.accent.clone()) } else { self.constants.get(key).cloned() },
                });
                set_arms.push(quote! {
                    "accent" => {
                        self.accent = value;
                        self.constants.remove("accent");
                    }
                });
            }
            "divider" => {
                get_arms.push(quote! {
                    "divider" => if !self.divider.is_empty() { Some(self.divider.clone()) } else { self.constants.get(key).cloned() },
                });
                set_arms.push(quote! {
                    "divider" => {
                        self.divider = value;
                        self.constants.remove("divider");
                    }
                });
            }
            _ => {}
        }
    }

    let expanded = quote! {
        pub trait ThemeItemExt {
            fn get_constant(&self, key: &str) -> Option<String>;
            fn set_constant(&mut self, key: &str, value: String);
            fn get_all_keys(&self) -> std::collections::HashSet<String>;
        }

        impl ThemeItemExt for ThemeItem {
            fn get_constant(&self, key: &str) -> Option<String> {
                match key {
                    #(#get_arms)*
                    _ => self.constants.get(key).cloned(),
                }
            }

            fn set_constant(&mut self, key: &str, value: String) {
                match key {
                    #(#set_arms)*
                    _ => {
                        self.constants.insert(key.to_string(), value);
                    }
                }
            }

            fn get_all_keys(&self) -> std::collections::HashSet<String> {
                let mut keys = std::collections::HashSet::new();
                #(#key_inserts)*
                for k in self.constants.keys() {
                    keys.insert(k.clone());
                }
                keys
            }
        }

        pub fn get_default_theme_item() -> ThemeItem {
            let mut item = ThemeItem::default();
            #(#default_setters)*
            item
        }
    };

    expanded.into()
}

struct ThemeApplyInput {
    path: syn::LitStr,
    theme_global: syn::Expr,
    name: syn::Expr,
    set_color: syn::Expr,
    set_length: syn::Expr,
}

impl syn::parse::Parse for ThemeApplyInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let path: syn::LitStr = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let theme_global: syn::Expr = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let name: syn::Expr = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let set_color: syn::Expr = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let set_length: syn::Expr = input.parse()?;
        Ok(ThemeApplyInput {
            path,
            theme_global,
            name,
            set_color,
            set_length,
        })
    }
}

#[proc_macro]
pub fn generate_theme_apply(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as ThemeApplyInput);
    let path_str = input.path.value();
    let content = match read_file(&path_str) {
        Ok(c) => c,
        Err(e) => panic!("Failed to read slint constants file '{}': {:?}", path_str, e),
    };
    let properties = parse_slint_theme(&content);

    let theme_global = &input.theme_global;
    let name = &input.name;
    let set_color = &input.set_color;
    let set_length = &input.set_length;

    let mut arms = Vec::new();
    for (name, prop_type, _) in properties {
        let name_str = name.as_str();
        let setter = format_ident!("set_{}", name);
        if prop_type == "color" {
            arms.push(quote! {
                #name_str => #set_color(&|c| #theme_global.#setter(c)),
            });
        } else if prop_type == "length" {
            arms.push(quote! {
                #name_str => #set_length(&|l| #theme_global.#setter(l)),
            });
        }
    }

    let expanded = quote! {
        match #name {
            #(#arms)*
            _ => {}
        }
    };

    expanded.into()
}

#[proc_macro]
pub fn generate_theme_ui_helpers(input: TokenStream) -> TokenStream {
    let path_str = syn::parse_macro_input!(input as syn::LitStr).value();
    let content = match read_file(&path_str) {
        Ok(c) => c,
        Err(e) => panic!("Failed to read slint constants file '{}': {:?}", path_str, e),
    };
    let properties = parse_slint_theme(&content);

    let mut constant_pushers = Vec::new();

    for (name, prop_type, _) in properties {
        let name_str = name.as_str();
        if prop_type == "color" {
            constant_pushers.push(quote! {
                {
                    let val = theme_item.get_constant(#name_str).or_else(|| default_item.get_constant(#name_str)).unwrap_or_default();
                    let color = parse_color(&val).unwrap_or_default();
                    constants_vec.push(crate::ThemeConstant {
                        name: #name_str.into(),
                        value: val.into(),
                        is_color: true,
                        color_value: crate::RgbaColor {
                            r: color.red() as f32,
                            g: color.green() as f32,
                            b: color.blue() as f32,
                            a: color.alpha() as f32 / 255.0,
                        },
                    });
                }
            });
        } else if prop_type == "length" {
            constant_pushers.push(quote! {
                {
                    let val = theme_item.get_constant(#name_str).or_else(|| default_item.get_constant(#name_str)).unwrap_or_default();
                    let val_num = if val.ends_with("px") { &val[..val.len() - 2] } else { &val };
                    constants_vec.push(crate::ThemeConstant {
                        name: #name_str.into(),
                        value: val_num.into(),
                        is_color: false,
                        color_value: crate::RgbaColor {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        },
                    });
                }
            });
        }
    }

    let expanded = quote! {
        pub fn update_theme_constants_ui(main_window: &crate::MainWindow, theme: &themes_proto::moosync::types::ThemeDetails) {
            let theme_item = theme.get_theme_item_or_default();
            let default_item = types::prelude::get_default_theme_item();
            let mut constants_vec = Vec::new();

            #(#constant_pushers)*

            let model = slint::VecModel::default();
            for c in constants_vec {
                model.push(c);
            }
            main_window.set_theme_constants(slint::ModelRc::new(model));
        }
    };

    expanded.into()
}

fn read_file(path_str: &str) -> std::io::Result<String> {
    use std::path::PathBuf;
    
    // 1. Try manifest dir
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let path = PathBuf::from(manifest_dir).join(path_str);
        if path.exists() {
            return std::fs::read_to_string(path);
        }
    }
    
    // 2. Try current dir
    if let Ok(current_dir) = std::env::current_dir() {
        let path = current_dir.join(path_str);
        if path.exists() {
            return std::fs::read_to_string(path);
        }
        
        // 3. Try walking up to find WORKSPACE or Cargo.toml
        let mut dir = current_dir;
        loop {
            let workspace = dir.join("WORKSPACE");
            let workspace_bazel = dir.join("WORKSPACE.bazel");
            let cargo_toml = dir.join("Cargo.toml");
            if workspace.exists() || workspace_bazel.exists() || cargo_toml.exists() {
                let path = dir.join(path_str);
                if path.exists() {
                    return std::fs::read_to_string(path);
                }
            }
            if let Some(parent) = dir.parent() {
                dir = parent.to_path_buf();
            } else {
                break;
            }
        }
    }
    
    // 4. Just try path directly
    std::fs::read_to_string(path_str)
}

fn parse_slint_theme(content: &str) -> Vec<(String, String, String)> {
    let mut properties = Vec::new();
    let content = strip_comments(content);
    
    if let Some(theme_start) = content.find("global Theme") {
        if let Some(brace_start) = content[theme_start..].find('{') {
            let start_idx = theme_start + brace_start + 1;
            let mut brace_count = 1;
            let mut end_idx = start_idx;
            let chars: Vec<char> = content[start_idx..].chars().collect();
            for (i, c) in chars.iter().enumerate() {
                if *c == '{' {
                    brace_count += 1;
                } else if *c == '}' {
                    brace_count -= 1;
                    if brace_count == 0 {
                        end_idx = start_idx + i;
                        break;
                    }
                }
            }
            
            let theme_block = &content[start_idx..end_idx];
            for statement in theme_block.split(';') {
                let statement = statement.trim();
                if statement.is_empty() {
                    continue;
                }
                if let Some(prop_idx) = statement.find("property") {
                    let after_prop = &statement[prop_idx + 8..].trim();
                    if after_prop.starts_with('<') {
                        if let Some(type_end) = after_prop.find('>') {
                            let prop_type = after_prop[1..type_end].trim().to_string();
                            let after_type = after_prop[type_end + 1..].trim();
                            if let Some(colon_idx) = after_type.find(':') {
                                let prop_name = after_type[..colon_idx].trim().to_string();
                                let default_val = after_type[colon_idx + 1..].trim().to_string();
                                properties.push((prop_name, prop_type, default_val));
                            } else {
                                let prop_name = after_type.trim().to_string();
                                properties.push((prop_name, prop_type, String::new()));
                            }
                        }
                    }
                }
            }
        }
    }
    properties
}

fn strip_comments(content: &str) -> String {
    let mut result = String::new();
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let chars: Vec<char> = content.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if in_line_comment {
            if chars[i] == '\n' {
                in_line_comment = false;
                result.push('\n');
            }
        } else if in_block_comment {
            if i + 1 < chars.len() && chars[i] == '*' && chars[i+1] == '/' {
                in_block_comment = false;
                i += 1;
            }
        } else {
            if i + 1 < chars.len() && chars[i] == '/' && chars[i+1] == '/' {
                in_line_comment = true;
                i += 1;
            } else if i + 1 < chars.len() && chars[i] == '/' && chars[i+1] == '*' {
                in_block_comment = true;
                i += 1;
            } else {
                result.push(chars[i]);
            }
        }
        i += 1;
    }
    result
}
