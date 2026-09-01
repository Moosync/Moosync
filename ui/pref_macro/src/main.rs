use std::fs;

#[path = "shared.rs"]
mod shared;

fn main() {
    let workspace = std::env::var("BUILD_WORKSPACE_DIRECTORY").unwrap_or_else(|_| ".".to_string());
    let workspace_path = std::path::PathBuf::from(workspace);

    let scratch_dir = workspace_path.join("scratch");
    let _ = fs::create_dir_all(&scratch_dir);

    let paths_yaml_path = workspace_path.join("ui/slint/src/settings/paths_prefs.yaml");
    let paths_content = fs::read_to_string(&paths_yaml_path)
        .unwrap_or_else(|e| panic!("Could not read {:?}: {}", paths_yaml_path, e));
    let paths_ident = syn::Ident::new("paths_items", proc_macro2::Span::call_site());
    let paths_handler = syn::Ident::new("PathsPageHandler", proc_macro2::Span::call_site());
    let paths_expanded =
        shared::generate_expansion(&paths_content, &paths_ident, &paths_handler).to_string();
    let paths_expanded_path = scratch_dir.join("paths_items_expanded.rs");
    fs::write(&paths_expanded_path, paths_expanded).unwrap();

    let system_yaml_path = workspace_path.join("ui/slint/src/settings/system_prefs.yaml");
    let system_content = fs::read_to_string(&system_yaml_path)
        .unwrap_or_else(|e| panic!("Could not read {:?}: {}", system_yaml_path, e));
    let system_ident = syn::Ident::new("system_items", proc_macro2::Span::call_site());
    let system_handler = syn::Ident::new("SystemPageHandler", proc_macro2::Span::call_site());
    let system_expanded =
        shared::generate_expansion(&system_content, &system_ident, &system_handler).to_string();
    let system_expanded_path = scratch_dir.join("system_items_expanded.rs");
    fs::write(&system_expanded_path, system_expanded).unwrap();

    println!(
        "Expanded code successfully written to scratch/paths_items_expanded.rs and scratch/system_items_expanded.rs"
    );
}
