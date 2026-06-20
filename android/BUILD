load("@rules_android//rules:rules.bzl", "android_binary")
load("@rules_kotlin//kotlin:android.bzl", "kt_android_library")

package(default_visibility = ["//visibility:public"])

kt_android_library(
    name = "moosync_kt",
    srcs = glob(["src/**/*.kt"]),
    custom_package = "app.moosync.android",
)

# android_binary packages the Slint shared library into an APK.
# The rust_shared_library (//ui/slint:slint_app_android) provides CcInfo,
# which android_binary picks up automatically to include the .so in the APK.
# Platform is set via --android_platforms=//:x86_64 at build time.
android_binary(
    name = "moosync_apk",
    custom_package = "app.moosync.android",
    manifest = "AndroidManifest.xml",
    resource_files = glob(["res/**"]),
    deps = [
        ":moosync_kt",
        "//ui/slint:slint_app_android",
    ],
)
