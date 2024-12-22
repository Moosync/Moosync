use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn find_function_details_json(target_dir: &Path) -> Option<PathBuf> {
    for entry in fs::read_dir(target_dir).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(found) = find_function_details_json(&path) {
                return Some(found);
            }
        } else if path.ends_with("function_details.json") {
            return Some(path);
        }
    }

    None
}

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");
    let manifest_dir = Path::new(&manifest_dir);

    if let Some(file_path) = find_function_details_json(manifest_dir.join("target").as_path()) {
        println!(
            "cargo:rustc-env=TAURI_INVOKE_PROC_DIR={}",
            file_path.to_string_lossy()
        );
    } else {
        println!("cargo:warning=Could not find function_details.json");
    }
}
