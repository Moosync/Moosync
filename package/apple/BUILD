load("@rules_apple//apple:macos.bzl", "macos_application")

filegroup(
    name = "AppIcons",
    srcs = glob(["Resources/Assets.xcassets/**"]),
)

macos_application(
    name = "Moosync",
    app_icons = [":AppIcons"],
    bundle_id = "com.moosync.music",
    bundle_name = "Moosync",
    infoplists = ["Info.plist"],
    minimum_os_version = "10.15",
    version = "12.0.0",  # Or hardcoded string "1.0.0"
    deps = ["//tauri:moosync"],
)
