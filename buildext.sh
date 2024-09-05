#!/bin/sh
cd src-extensions-wasm
cargo build --release
cp target/release/wasm-extension-runner ../src-tauri/binaries/exthost-wasm-$(rustc -Vv | grep host | cut -f2 -d' ')
