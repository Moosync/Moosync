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

use leptos_i18n_build::{ParseOptions, TranslationsInfos};
use pref_gen::generate_components;
use std::{
    env, fs,
    path::{Path, PathBuf},
};
use tauri_invoke_proc_ui::generate_tauri_invoke_wrapper;

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
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");

    let i18n_mod_directory = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("i18n");

    let options = ParseOptions::default().interpolate_display(true);
    eprintln!("Using options: {:?}", env!("CARGO_MANIFEST_DIR"));

    let translations_infos = TranslationsInfos::parse(options).unwrap();

    translations_infos.rerun_if_locales_changed();

    translations_infos
        .generate_i18n_module(i18n_mod_directory)
        .unwrap();

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");
    let manifest_dir = Path::new(&manifest_dir);

    if let Some(file_path) = find_function_details_json(manifest_dir) {
        generate_tauri_invoke_wrapper(
            &file_path,
            vec![
                "types".into(),
                "themes_proto".into(),
                "songs_proto".into(),
                "extensions_proto".into(),
                "ui_proto".into(),
            ],
        );
    } else {
        panic!("Could not find function_details.json {:?}", manifest_dir);
    }

    generate_components("prefs.yaml");

    if cfg!(debug_assertions) {
        println!("cargo:rustc-cfg=erase_components");
    }
}
