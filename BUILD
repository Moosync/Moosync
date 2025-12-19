load("@gazelle//:def.bzl", "gazelle")

# gazelle:exclude ui
# gazelle:exclude core/types
# gazelle:exclude core/database
# gazelle:exclude core/extensions
# gazelle:exclude core/file_scanner
# gazelle:exclude core/librespot
# gazelle:exclude core/lyrics
# gazelle:exclude core/mpris
# gazelle:exclude core/preferences
# gazelle:exclude tauri-invoke-proc

load("@rules_rust_wasm_bindgen//:defs.bzl", "rust_wasm_bindgen_toolchain")

gazelle(
    name = "gazelle",
    gazelle = "@gazelle_rust//:gazelle_bin",
)

rust_wasm_bindgen_toolchain(
    name = "wasm_bindgen_toolchain_impl",
    wasm_bindgen_cli = "@bindeps//:wasm-bindgen-cli__wasm-bindgen",
)

toolchain(
    name = "wasm_bindgen_toolchain",
    toolchain = "wasm_bindgen_toolchain_impl",
    toolchain_type = "@rules_rust_wasm_bindgen//:toolchain_type",
)
