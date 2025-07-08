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

    if let Some(file_path) = find_function_details_json(manifest_dir.join("src-tauri").as_path()) {
        println!(
            "cargo:rustc-env=TAURI_INVOKE_PROC_DIR={}",
            file_path.to_string_lossy()
        );
    } else {
        println!("cargo:warning=Could not find function_details.json");
    }
    if cfg!(debug_assertions) {
        println!("cargo:rustc-cfg=erase_components");
    }
}
