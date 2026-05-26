load("@rules_pkg//pkg:mappings.bzl", "pkg_files")
load("@rules_pkg//pkg:zip.bzl", "pkg_zip")

package(default_visibility = ["//visibility:public"])

# Copies the APK output from the android_binary target into packages/android/
pkg_files(
    name = "moosync_apk_files",
    srcs = ["//android:moosync_apk"],
    prefix = "",
)

pkg_zip(
    name = "moosync_android_package",
    srcs = [":moosync_apk_files"],
    out = "moosync-android.zip",
)
