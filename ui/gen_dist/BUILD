load("@aspect_rules_rollup//rollup:defs.bzl", "rollup")
load("@npm//:defs.bzl", "npm_link_all_packages")
load("//tools:defs.bzl", "html_page")

package(default_visibility = ["//ui:__subpackages__"])

npm_link_all_packages(name = "node_modules")

rollup(
    name = "moosync_ui_bundle",
    srcs = [
        "//ui/gen_dist/styles:bundle",
        "//ui/leptos:moosync_ui_bindgen",
    ],
    config_file = ":rollup.config.js",
    entry_point = "main.js",
    node_modules = ":node_modules",
    deps = [
        ":node_modules/@rollup/plugin-node-resolve",
        ":node_modules/@tauri-apps/api",
    ],
)

html_page(
    name = "moosync_dist",
    css = [
        "//ui/gen_dist/styles:bundle",
    ],
    fonts = ["//ui/gen_dist/fonts"],
    js = [
        ":moosync_ui_bundle",
    ],
    public_path = "/",
    title = "Moosync",
    wasm = [
        "//ui/leptos:moosync_ui_bindgen",
    ],
)
