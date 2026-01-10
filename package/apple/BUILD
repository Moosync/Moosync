load("@rules_pkg//pkg:mappings.bzl", "pkg_attributes", "pkg_files")
load("@rules_pkg//pkg:zip.bzl", "pkg_zip")
load("//:version.bzl", "APP_NAME", "RELEASE", "VERSION")

genrule(
    name = "rename_binary",
    srcs = ["//tauri:moosync"],
    outs = ["Moosync"],
    cmd = "cp $< $@",
    executable = True,
    target_compatible_with = ["@platforms//os:macos"],
)

genrule(
    name = "process_info_plist",
    srcs = ["Info.plist"],
    outs = ["Info_processed.plist"],
    cmd = "sed -e 's/{{VERSION}}/{ver}/g' -e 's/{{RELEASE}}/{rel}/g' $< > $@".format(
        rel = RELEASE,
        ver = VERSION,
    ),
    target_compatible_with = ["@platforms//os:macos"],
)

pkg_files(
    name = "app_binary",
    srcs = [":rename_binary"],
    attributes = pkg_attributes(mode = "0755"),
    prefix = "Moosync.app/Contents/MacOS",
    target_compatible_with = ["@platforms//os:macos"],
)

pkg_files(
    name = "app_plist",
    srcs = [":process_info_plist"],
    prefix = "Moosync.app/Contents",
    renames = {
        "Info_processed.plist": "Info.plist",
    },
    target_compatible_with = ["@platforms//os:macos"],
)

pkg_files(
    name = "app_resources",
    srcs = glob(["Resources/**/*"]),
    prefix = "Moosync.app/Contents/Resources",
    strip_prefix = "Resources",
    target_compatible_with = ["@platforms//os:macos"],
)

pkg_zip(
    name = "moosync_mac_raw",
    srcs = [
        ":app_binary",
        ":app_plist",
        ":app_resources",
    ],
    out = "moosync_raw.zip",
    target_compatible_with = ["@platforms//os:macos"],
)

genrule(
    name = "rename_mac_arm64",
    srcs = [":moosync_mac_raw"],
    outs = ["{}-{}-{}.darwin-arm64.zip".format(APP_NAME, VERSION, RELEASE)],
    cmd = "cp $< $@",
    target_compatible_with = ["@platforms//os:macos"],
)

genrule(
    name = "rename_mac_x64",
    srcs = [":moosync_mac_raw"],
    outs = ["{}-{}-{}.darwin-x64.zip".format(APP_NAME, VERSION, RELEASE)],
    cmd = "cp $< $@",
    target_compatible_with = ["@platforms//os:macos"],
)

filegroup(
    name = "apple_release",
    srcs = select({
        "@platforms//cpu:arm64": [":rename_mac_arm64"],
        "@platforms//cpu:x86_64": [":rename_mac_x64"],
        "//conditions:default": [":rename_mac_x64"],
    }),
    target_compatible_with = ["@platforms//os:macos"],
    visibility = ["//visibility:public"],
)
