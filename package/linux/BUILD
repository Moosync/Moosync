load("@rpmpack//:def.bzl", "pkg_tar2rpm")
load("@rules_pkg//pkg:mappings.bzl", "pkg_attributes", "pkg_files")
load("@rules_pkg//pkg:deb.bzl", "pkg_deb")
load("@rules_pkg//pkg:tar.bzl", "pkg_tar")
load("//:version.bzl", "APP_NAME", "RELEASE", "VERSION")

pkg_files(
    name = "moosync_binary",
    srcs = ["//ui/slint:slint_app"],
    attributes = pkg_attributes(mode = "0755"),
    prefix = "/usr/bin",
    renames = {
        "slint_app": "moosync",
    },
)

pkg_files(
    name = "moosync_desktop",
    srcs = ["moosync.desktop"],
    attributes = pkg_attributes(mode = "0644"),
    prefix = "/usr/share/applications",
)

pkg_tar(
    name = "moosync_filesystem_tar",
    srcs = [":moosync_binary", ":moosync_desktop"],
    deps = ["//package:moosync_icons_tar"],
)

pkg_tar2rpm(
    name = "moosync_rpm_raw",
    arch = select({
        "@platforms//cpu:arm64": "aarch64",
        "@platforms//cpu:x86_64": "x86_64",
        "//conditions:default": "x86_64",
    }),
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
    architecture = select({
        "@platforms//cpu:arm64": "arm64",
        "@platforms//cpu:x86_64": "amd64",
        "//conditions:default": "amd64",
    }),
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
    ],
    outs = [
        "{}-{}-{}.x86_64.rpm".format(APP_NAME, VERSION, RELEASE),
        "{}-{}-{}.amd64.deb".format(APP_NAME, VERSION, RELEASE),
    ],
    cmd = """
    cp $(location :moosync_rpm_raw) $(location {name}-{ver}-{rel}.x86_64.rpm) && \
    cp $(location :moosync_deb_raw) $(location {name}-{ver}-{rel}.amd64.deb)
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
    ],
    outs = [
        "{}-{}-{}.aarch64.rpm".format(APP_NAME, VERSION, RELEASE),
        "{}-{}-{}.arm64.deb".format(APP_NAME, VERSION, RELEASE),
    ],
    cmd = """
    cp $(location :moosync_rpm_raw) $(location {name}-{ver}-{rel}.aarch64.rpm) && \
    cp $(location :moosync_deb_raw) $(location {name}-{ver}-{rel}.arm64.deb)
    """.format(
        name = APP_NAME,
        rel = RELEASE,
        ver = VERSION,
    ),
)

filegroup(
    name = "linux_release",
    srcs = select({
        "@platforms//cpu:arm64": [":rename_arm64"],
        "//conditions:default": [":rename_x64"],
    }),
    visibility = ["//visibility:public"],
)
