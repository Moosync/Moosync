load("@rules_android//rules:rules.bzl", "android_binary")

package(default_visibility = ["//visibility:public"])

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
        "//ui/slint:slint_app_android",
    ],
)
