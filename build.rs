use std::{env, path::Path};

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");
    let manifest_dir = Path::new(&manifest_dir);
    let profile = env::var("PROFILE").expect("PROFILE environment variable is not set");

    let file_path = manifest_dir.join("target").join(profile).join("build");

    println!(
        "cargo:rustc-env=TAURI_INVOKE_PROC_DIR={}",
        file_path.to_string_lossy()
    );
}
