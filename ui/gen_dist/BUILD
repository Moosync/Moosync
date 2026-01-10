load("@aspect_rules_rollup//rollup:defs.bzl", "rollup")
load("@npm//:defs.bzl", "npm_link_all_packages")
load("//tools:defs.bzl", "html_page")

package(default_visibility = ["//ui:__subpackages__"])

npm_link_all_packages(name = "node_modules")

genrule(
    name = "generate_entry",
    srcs = [
        "main.js.in",
        "//ui/leptos:moosync_ui_bindgen",
    ],
    outs = ["main.js"],
    cmd = """
        JS_FILE=""
        for f in $(SRCS); do
            if [[ "$$f" == *.js ]]; then
                JS_FILE="$$f"
                break
            fi
        done

        if [ -z "$$JS_FILE" ]; then
            echo "Error: Could not find .js file in sources" >&2; exit 1
        fi

        PYTHON_BIN="$(PYTHON3)"
        REL_PATH=$$(python3 -c "import os, pathlib; print(pathlib.Path(os.path.relpath('$$JS_FILE', os.path.dirname('$@'))).as_posix())")
        
        sed "s|@JS_ENTRY@|./$$REL_PATH|g" $(location main.js.in) > $@
    """,
    toolchains = ["@rules_python//python:current_py_toolchain"],
)

rollup(
    name = "moosync_ui_bundle",
    srcs = [
        "//ui/gen_dist/styles:bundle",
        "//ui/leptos:moosync_ui_bindgen",
    ],
    config_file = ":rollup.config.js",
    entry_point = ":generate_entry",
    node_modules = ":node_modules",
    deps = [
        ":node_modules/@rollup/plugin-node-resolve",
        ":node_modules/@tauri-apps/api",
        ":node_modules/rollup",
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
