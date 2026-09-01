extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{
    Ident, LitStr, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

#[path = "shared.rs"]
mod shared;

struct MacroInput {
    yaml_path: LitStr,
    property_name: Ident,
    handler_name: Ident,
}

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let yaml_path: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;
        let property_name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let handler_name: Ident = input.parse()?;
        Ok(MacroInput {
            yaml_path,
            property_name,
            handler_name,
        })
    }
}

#[proc_macro]
pub fn generate_preferences(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as MacroInput);
    let yaml_path = input.yaml_path.value();
    let property_name = &input.property_name;
    let handler_name = &input.handler_name;

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let mut path = std::path::PathBuf::from(&manifest_dir).join(&yaml_path);
    if !path.exists() {
        path = std::path::PathBuf::from(&yaml_path);
    }
    if !path.exists() {
        path = std::path::PathBuf::from("ui/slint").join(&yaml_path);
    }
    if !path.exists() {
        path = std::path::PathBuf::from("ui/slint/src/settings")
            .join(std::path::Path::new(&yaml_path).file_name().unwrap());
    }

    if !path.exists() {
        panic!(
            "Could not find YAML preference file: {} (resolved path: {:?}, manifest_dir: {:?})",
            yaml_path, path, manifest_dir
        );
    }

    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read YAML file at {:?}: {}", path, e));

    let expanded = shared::generate_expansion(&content, property_name, handler_name);

    TokenStream::from(expanded)
}
