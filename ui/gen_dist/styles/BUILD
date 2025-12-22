load("@gzgz_rules_sass//sass:defs.bzl", "sass_binary", "sass_library")

package(default_visibility = ["//visibility:public"])

sass_library(
    name = "bootstrap",
    srcs = glob(["bootstrap/scss/**/*.scss"]),
)

sass_library(
    name = "app_components",
    srcs = [
        "allsongs.scss",
        "color_picker.scss",
        "context_menu.scss",
        "login_modal.scss",
        "mobile.scss",
        "musicbar.scss",
        "musicinfo.scss",
        "now_playing.scss",
        "playlists.scss",
        "prefs.scss",
        "routes.scss",
        "scrollbar.scss",
        "sidebar.scss",
        "themes.scss",
        "topbar.scss",
        "transitions.scss",
    ],
    deps = [":bootstrap"],
)

sass_binary(
    name = "bundle",
    src = "global.scss",
    output_name = "app_bundle.css",
    output_style = "compressed",
    sourcemap = True,
    deps = [
        ":app_components",
        ":bootstrap",
    ],
)
