load("@rules_android//rules:rules.bzl", "android_binary")
load("@rules_cc//cc:defs.bzl", "cc_import")
load("@rules_kotlin//kotlin:android.bzl", "kt_android_library", "kt_android_local_test")
load("@rules_kotlin//kotlin:jvm.bzl", "kt_jvm_import", "kt_jvm_library", "kt_jvm_test")
load("@rules_python//python:defs.bzl", "py_binary", "py_test")
load("@rules_shell//shell:sh_binary.bzl", "sh_binary")

package(default_visibility = ["//visibility:public"])

kt_android_library(
    name = "moosync_kt",
    srcs = glob(["src/**/*.kt"]),
    custom_package = "app.moosync.android",
    manifest = "AndroidManifest.xml",
    resource_files = glob(["res/**"]),
    deps = [
        "//core/types/protos:android_scanner_java_proto_lite",
        "//core/types/protos:songs_java_proto_lite",
    ],
)

kt_jvm_import(
    name = "android_sdk_jar",
    jars = ["@androidsdk//:platforms/android-34/android.jar"],
)

kt_jvm_library(
    name = "scanner_jvm_lib",
    srcs = glob(["src/app/moosync/android/scanner/*.kt"]),
    deps = [
        ":android_sdk_jar",
        "//core/types/protos:android_scanner_java_proto_lite",
        "//core/types/protos:songs_java_proto_lite",
    ],
)

kt_jvm_test(
    name = "scanner_kt_test",
    srcs = ["test/app/moosync/android/scanner/ScannerTest.kt"],
    test_class = "app.moosync.android.scanner.ScannerTest",
    deps = [
        ":scanner_jvm_lib",
        "//core/types/protos:android_scanner_java_proto_lite",
        "//core/types/protos:songs_java_proto_lite",
        "@maven//:junit_junit",
    ],
)

kt_android_local_test(
    name = "handlers_test",
    srcs = ["test/app/moosync/android/HandlerUnitTests.kt"],
    custom_package = "app.moosync.android",
    manifest = "AndroidManifest.xml",
    resource_files = glob(["res/**"]),
    test_class = "app.moosync.android.HandlerUnitTests",
    deps = [
        ":moosync_kt",
        "@maven//:junit_junit",
        "@maven//:org_mockito_mockito_core",
    ],
)

kt_android_local_test(
    name = "native_bridge_test",
    srcs = ["test/app/moosync/android/NativeBridgeIntegrationTest.kt"],
    custom_package = "app.moosync.android",
    manifest = "AndroidManifest.xml",
    resource_files = glob(["res/**"]),
    test_class = "app.moosync.android.NativeBridgeIntegrationTest",
    deps = [
        ":moosync_kt",
        "@maven//:junit_junit",
        "@maven//:org_mockito_mockito_core",
    ],
)

kt_android_local_test(
    name = "media_service_test",
    srcs = ["test/app/moosync/android/services/MediaServiceLifecycleTest.kt"],
    custom_package = "app.moosync.android",
    manifest = "AndroidManifest.xml",
    resource_files = glob(["res/**"]),
    test_class = "app.moosync.android.services.MediaServiceLifecycleTest",
    deps = [
        ":moosync_kt",
        "@maven//:junit_junit",
        "@maven//:org_mockito_mockito_core",
    ],
)

test_suite(
    name = "android_tests",
    tests = [
        ":handlers_test",
        ":media_service_test",
        ":native_bridge_test",
        ":scanner_kt_test",
    ],
)

cc_import(
    name = "libcxx_shared",
    shared_library = select({
        "@platforms//cpu:x86_64": "@androidndk//toolchains/llvm/prebuilt/linux-x86_64/sysroot:usr/lib/x86_64-linux-android/libc++_shared.so",
        "@platforms//cpu:aarch64": "@androidndk//toolchains/llvm/prebuilt/linux-x86_64/sysroot:usr/lib/aarch64-linux-android/libc++_shared.so",
        "//conditions:default": "@androidndk//toolchains/llvm/prebuilt/linux-x86_64/sysroot:usr/lib/x86_64-linux-android/libc++_shared.so",
    }),
)

py_binary(
    name = "extract_ld_linux",
    srcs = ["extract_ld_linux.py"],
    deps = ["@extensions_sdk//tools:elf_align_lib"],
)

py_test(
    name = "extract_ld_linux_test",
    srcs = ["extract_ld_linux_test.py"],
    deps = [
        ":extract_ld_linux",
        "@extensions_sdk//tools:elf_align_lib",
    ],
)

genrule(
    name = "extract_ld_linux_x86_64",
    srcs = ["@termux_glibc_x86_64//file"],
    outs = ["x86_64/libld_linux.so"],
    cmd = "$(location :extract_ld_linux) --archive $< --member data/data/com.termux/files/usr/glibc/lib/ld-linux-x86-64.so.2 --out $@ --align-16k",
    tools = [":extract_ld_linux"],
)

genrule(
    name = "extract_ld_linux_arm64",
    srcs = ["@termux_glibc_arm64//file"],
    outs = ["arm64/libld_linux.so"],
    cmd = "$(location :extract_ld_linux) --archive $< --member data/data/com.termux/files/usr/glibc/lib/ld-linux-aarch64.so.1 --out $@",
    tools = [":extract_ld_linux"],
)

cc_import(
    name = "libld_linux",
    shared_library = select({
        "@platforms//cpu:x86_64": ":extract_ld_linux_x86_64",
        "@platforms//cpu:aarch64": ":extract_ld_linux_arm64",
        "//conditions:default": ":extract_ld_linux_x86_64",
    }),
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
        ":libcxx_shared",
        ":libld_linux",
        ":moosync_kt",
        "//ui/slint:slint_app_android",
    ],
)

sh_binary(
    name = "run_emulator",
    srcs = ["run_emulator.sh"],
    data = [
        ":moosync_apk",
    ],
)
