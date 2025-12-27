load("@rpmpack//:def.bzl", "pkg_tar2rpm")
load("@rules_appimage//appimage:appimage.bzl", "appimage")
load("@rules_pkg//pkg:deb.bzl", "pkg_deb")
load("@rules_pkg//pkg:mappings.bzl", "pkg_files")
load("@rules_pkg//pkg:tar.bzl", "pkg_tar")
load("//tools:defs.bzl", "dirgroup")

dirgroup(
    name = "icons",
    srcs = glob([
        "icons/**",
    ]),
    visibility = ["//visibility:public"],
)

pkg_tar(
    name = "moosync_bin_tar",
    srcs = ["//tauri:moosync"],
    mode = "0755",
    package_dir = "/usr/bin",
)

pkg_tar(
    name = "moosync_desktop_tar",
    srcs = ["moosync.desktop"],
    mode = "0644",
    package_dir = "/usr/share/applications",
)

pkg_files(
    name = "moosync_icon_mapping",
    srcs = [
        "icons/128x128.png",
        "icons/32x32.png",
        "icons/icon.png",
    ],
    renames = {
        "icons/32x32.png": "/usr/share/icons/hicolor/32x32/apps/moosync.png",
        "icons/128x128.png": "/usr/share/icons/hicolor/128x128/apps/moosync.png",
        "icons/icon.png": "/usr/share/pixmaps/moosync.png",
    },
)

pkg_tar(
    name = "moosync_icons_tar",
    srcs = [":moosync_icon_mapping"],
    mode = "0644",
)

pkg_tar(
    name = "moosync_filesystem_tar",
    deps = [
        ":moosync_bin_tar",
        ":moosync_desktop_tar",
        ":moosync_icons_tar",
    ],
)

appimage(
    name = "moosync_appimage",
    binary = "//tauri:moosync",
)

pkg_tar2rpm(
    name = "moosync_rpm",
    arch = "x86_64",
    data = ":moosync_filesystem_tar",
    pkg_name = "moosync",
    release = "1",
    requires = [
        "openssl-libs",
        "webkit2gtk3",
    ],
    version = "12.0.0",
)

pkg_deb(
    name = "moosync_deb",
    architecture = "amd64",
    built_using = "bazel",
    data = ":moosync_filesystem_tar",
    depends = [
        "libwebkit2gtk-4.0-37",
        "libssl3",
    ],
    description = "Moosync",
    maintainer = "Sahil Gupte <ovenoboyo@gmail.com>",
    package = "moosync",
    version = "12.0.0",
)

pkg_tar(
    name = "moosync_tar",
    extension = "tar.gz",
    deps = [
        ":moosync_bin_tar",
        ":moosync_desktop_tar",
        ":moosync_icons_tar",
    ],
)
