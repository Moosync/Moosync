load("//:version.bzl", "APP_NAME", "RELEASE", "VERSION")

genrule(
    name = "moosync_installer_raw",
    srcs = [
        "moosync.nsi",
        "//ui/slint:slint_app",
    ],
    outs = ["MoosyncSetup.exe"],
    cmd = """
    if command -v cygpath >/dev/null 2>&1; then
      BINARY_ABS_PATH=$$(cygpath -w "$$PWD/$(location //ui/slint:slint_app)")
      OUT_PATH=$$(cygpath -w "$$PWD/$@")
      makensis \
        -V4 \
        -DBINARY_PATH="$$BINARY_ABS_PATH" \
        "$(location moosync.nsi)"
      mv package/windows/MoosyncSetup.exe "$$(cygpath -u "$$OUT_PATH")"
    else
      makensis \
        -V4 \
        -DBINARY_PATH="$$PWD/$(location //ui/slint:slint_app)" \
        "$(location moosync.nsi)"
      mv package/windows/MoosyncSetup.exe "$@"
    fi
    """,
    target_compatible_with = ["@platforms//os:windows"],
)

genrule(
    name = "moosync_portable_raw",
    srcs = ["//ui/slint:slint_app"],
    outs = ["Moosync.exe"],
    cmd = "cp $< $@",
    executable = True,
    target_compatible_with = ["@platforms//os:windows"],
)

genrule(
    name = "rename_windows",
    srcs = [
        ":moosync_installer_raw",
        ":moosync_portable_raw",
    ],
    outs = [
        "{}-{}-{}.x86_64-setup.exe".format(APP_NAME, VERSION, RELEASE),
        "{}-{}-{}.x86_64.exe".format(APP_NAME, VERSION, RELEASE),
    ],
    cmd = """
    cp $(location :moosync_installer_raw) $(location {name}-{ver}-{rel}.x86_64-setup.exe) && \
    cp $(location :moosync_portable_raw) $(location {name}-{ver}-{rel}.x86_64.exe)
    """.format(
        name = APP_NAME,
        rel = RELEASE,
        ver = VERSION,
    ),
    target_compatible_with = ["@platforms//os:windows"],
)

filegroup(
    name = "windows_release",
    srcs = [":rename_windows"],
    target_compatible_with = ["@platforms//os:windows"],
    visibility = ["//visibility:public"],
)
