load("//:version.bzl", "APP_NAME", "RELEASE", "VERSION")

genrule(
    name = "moosync_installer_raw",
    srcs = [
        "moosync.nsi",
        "//tauri:moosync",
    ],
    outs = ["MoosyncSetup.exe"],
    cmd = """
    ls -la $$PWD/$(location //tauri:moosync)
    BINARY_ABS_PATH=$$(cygpath -w "$$PWD/$(location //tauri:moosync)")
    OUT_PATH=$$(cygpath -w "$$PWD/$@")
    makensis \
      -V4 \
      -DBINARY_PATH="$$BINARY_ABS_PATH" \
      -DOUTFILE="$$OUT_PATH" \
      "$(location moosync.nsi)"

    ls -la "$$(cygpath -u "$$OUT_PATH")"
    """,
)

genrule(
    name = "moosync_portable_raw",
    srcs = ["//tauri:moosync"],
    outs = ["Moosync.exe"],
    cmd = "cp $< $@",
    executable = True,
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
)

filegroup(
    name = "windows_release",
    srcs = [":rename_windows"],
    visibility = ["//visibility:public"],
)
