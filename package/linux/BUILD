load("@rpmpack//:def.bzl", "pkg_tar2rpm")
load("@rules_pkg//pkg:deb.bzl", "pkg_deb")
load("@rules_pkg//pkg:tar.bzl", "pkg_tar")
load("//:version.bzl", "APP_NAME", "RELEASE", "VERSION")

config_setting(
    name = "cpu_arm64",
    values = {"cpu": "aarch64"},
)

ARCH_DEB = select({
    "@platforms//cpu:arm64": "arm64",
    "@platforms//cpu:x86_64": "amd64",
    "//conditions:default": "amd64",
})

ARCH_RPM = select({
    "@platforms//cpu:arm64": "aarch64",
    "@platforms//cpu:x86_64": "x86_64",
    "//conditions:default": "x86_64",
})

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

pkg_tar(
    name = "moosync_filesystem_tar",
    deps = [
        ":moosync_bin_tar",
        ":moosync_desktop_tar",
        "//package:moosync_icons_tar",
    ],
)

genrule(
    name = "moosync_appimage_raw",
    srcs = [
        "//tauri:moosync",  # Your binary
        "moosync.desktop",  # Your desktop entry
        "//package:default_icon",
    ],
    outs = ["Moosync.AppImage"],
    cmd = """
    cp $(location @linuxdeploy//file) linuxdeploy
    chmod +x linuxdeploy
    
    cp $(location @linuxdeploy_plugin_gtk//file) linuxdeploy-plugin-gtk.sh
    chmod +x linuxdeploy-plugin-gtk.sh
    
    ./linuxdeploy --appimage-extract > /dev/null
    
    LD_TOOL=./squashfs-root/AppRun

    export PATH=.:$$PATH
    export DEPLOY_GTK_VERSION=3
    export NO_STRIP=true

    cp $(location //package:default_icon) moosync.png
    
    $$LD_TOOL --appdir AppDir \
        --executable $(location //tauri:moosync) \
        --desktop-file $(location moosync.desktop) \
        --icon-file moosync.png \
        --plugin gtk \
        --output appimage

    mv *.AppImage $@
    """,
    tools = [
        "@linuxdeploy//file",
        "@linuxdeploy_plugin_gtk//file",
    ],
)

pkg_tar2rpm(
    name = "moosync_rpm_raw",
    arch = ARCH_RPM,
    data = ":moosync_filesystem_tar",
    pkg_name = "moosync",
    release = RELEASE,
    requires = [
        "webkit2gtk3 openssl",
    ],
    version = VERSION,
)

pkg_deb(
    name = "moosync_deb_raw",
    architecture = ARCH_DEB,
    built_using = "bazel",
    data = ":moosync_filesystem_tar",
    depends = [
        "libwebkit2gtk-4.0-37",
        "libssl3",
    ],
    description = "Moosync",
    maintainer = "Sahil Gupte <ovenoboyo@gmail.com>",
    package = "moosync",
    version = VERSION,
)

genrule(
    name = "rename_x64",
    srcs = [
        ":moosync_rpm_raw",
        ":moosync_deb_raw",
        ":moosync_appimage_raw",
    ],
    outs = [
        "{}-{}-{}.x86_64.rpm".format(APP_NAME, VERSION, RELEASE),
        "{}-{}-{}.amd64.deb".format(APP_NAME, VERSION, RELEASE),
        "{}-{}-{}.x86_64.AppImage".format(APP_NAME, VERSION, RELEASE),
    ],
    cmd = """
    cp $(location :moosync_rpm_raw) $(location {name}-{ver}-{rel}.x86_64.rpm) && \
    cp $(location :moosync_deb_raw) $(location {name}-{ver}-{rel}.amd64.deb) && \
    cp $(location :moosync_appimage_raw) $(location {name}-{ver}-{rel}.x86_64.AppImage)
    """.format(
        name = APP_NAME,
        rel = RELEASE,
        ver = VERSION,
    ),
)

genrule(
    name = "rename_arm64",
    srcs = [
        ":moosync_rpm_raw",
        ":moosync_deb_raw",
        ":moosync_appimage_raw",
    ],
    outs = [
        "{}-{}-{}.aarch64.rpm".format(APP_NAME, VERSION, RELEASE),
        "{}-{}-{}.arm64.deb".format(APP_NAME, VERSION, RELEASE),
        "{}-{}-{}.aarch64.AppImage".format(APP_NAME, VERSION, RELEASE),
    ],
    cmd = """
    cp $(location :moosync_rpm_raw) $(location {name}-{ver}-{rel}.aarch64.rpm) && \
    cp $(location :moosync_deb_raw) $(location {name}-{ver}-{rel}.arm64.deb) && \
    cp $(location :moosync_appimage_raw) $(location {name}-{ver}-{rel}.aarch64.AppImage)
    """.format(
        name = APP_NAME,
        rel = RELEASE,
        ver = VERSION,
    ),
)

filegroup(
    name = "linux_release",
    srcs = select({
        ":cpu_arm64": [":rename_arm64"],
        "//conditions:default": [":rename_x64"],
    }),
    visibility = ["//visibility:public"],
)
