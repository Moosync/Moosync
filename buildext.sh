#!/bin/sh
cd lib/extensions-wasm
rm -rf ../../src-tauri/binaries/exthost-wasm-*
cargo build --release
mkdir -p ../../src-tauri/binaries
cp target/release/wasm-extension-runner ../../src-tauri/binaries/exthost-wasm-$(rustc -Vv | grep host | cut -f2 -d' ')
