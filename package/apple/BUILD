load("@rules_pkg//pkg:mappings.bzl", "pkg_attributes", "pkg_files")
load("@rules_pkg//pkg:zip.bzl", "pkg_zip")

genrule(
    name = "rename_binary",
    srcs = ["//tauri:moosync"],
    outs = ["Moosync"],
    cmd = "cp $< $@",
    executable = True,
)

pkg_files(
    name = "app_binary",
    srcs = [":rename_binary"],
    attributes = pkg_attributes(mode = "0755"),
    prefix = "Moosync.app/Contents/MacOS",
)

# 2. Place Info.plist into Contents/
pkg_files(
    name = "app_plist",
    srcs = ["Info.plist"],
    prefix = "Moosync.app/Contents",
)

pkg_files(
    name = "app_resources",
    srcs = glob(["Resources/**"]),
    prefix = "Moosync.app/Contents/Resources",
    strip_prefix = "Resources",
)

pkg_zip(
    name = "moosync_mac_app",
    srcs = [
        ":app_binary",
        ":app_plist",
        ":app_resources",
    ],
    out = "Moosync.zip",
)
