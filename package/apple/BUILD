load("@rules_apple//apple:macos.bzl", "macos_application")
load("@rules_apple//apple:versioning.bzl", "apple_bundle_version")

filegroup(
    name = "AppIcons",
    srcs = glob(["Resources/Assets.xcassets/**"]),
)

apple_bundle_version(
    name = "moosync_version",
    build_version = "12.0.0",
    short_version_string = "12.0",
)

macos_application(
    name = "Moosync",
    app_icons = [":AppIcons"],
    bundle_id = "com.moosync.music",
    bundle_name = "Moosync",
    infoplists = ["Info.plist"],
    minimum_os_version = "10.15",
    version = ":moosync_version",  # Or hardcoded string "1.0.0"
    deps = ["//tauri:moosync"],
)
