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

use std::path::Path;
use std::process::Command;
use std::{env, fs};

fn main() {
    let out_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let ffmpeg_dir = Path::new(&out_dir).join("ffmpeg");
    let ffmpeg_build_dir = ffmpeg_dir.join("build");

    if !ffmpeg_dir.exists() {
        Command::new("git")
            .args([
                "clone",
                "https://github.com/ffmpeg/ffmpeg",
                "--depth",
                "1",
                "--single-branch",
                "--branch",
                "release/7.0",
                ffmpeg_dir.to_str().unwrap(),
            ])
            .status()
            .unwrap();
    }

    if !ffmpeg_build_dir.exists() {
        fs::create_dir(&ffmpeg_build_dir).unwrap();
        let prefix = ffmpeg_build_dir.join("build");
        let mut configure = Command::new("../configure");
        configure.current_dir(&ffmpeg_build_dir);
        configure.arg(format!("--prefix={}", prefix.to_str().unwrap()));

        configure.status().unwrap();

        Command::new("make")
            .arg("-j")
            .arg(num_cpus::get().to_string())
            .current_dir(&ffmpeg_build_dir)
            .status()
            .unwrap();

        Command::new("make")
            .arg("install")
            .current_dir(&ffmpeg_build_dir)
            .status()
            .unwrap();
    }

    let ffmpeg_pkg_config_path = ffmpeg_build_dir.join("build").join("lib").join("pkgconfig");

    env::set_var(
        "FFMPEG_PKG_CONFIG_PATH",
        ffmpeg_pkg_config_path.to_str().unwrap(),
    );
    env::set_var(
        "FFMPEG_INCLUDE_DIR",
        ffmpeg_build_dir
            .join("build")
            .join("include")
            .to_str()
            .unwrap(),
    );

    let target_os = env::var("CARGO_CFG_TARGET_OS");
    if let Ok("android") = target_os.as_ref().map(|x| &**x) {
        println!("cargo:rustc-link-lib=dylib=stdc++");
        println!("cargo:rustc-link-lib=c++_shared");
    }
}
