load("@rules_rust//cargo:defs.bzl", "cargo_build_script")
load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library", "rust_shared_library")
load("//:defs.bzl", "platform_transition_rule")

package(default_visibility = ["//visibility:public"])

cargo_build_script(
    name = "build_script",
    srcs = ["build.rs"],
    data = ["src/app.slint"],
    deps = ["@crates//:slint-build"],  # Make sure slint-build is in your Cargo.toml/crates list
)

rust_library(
    name = "slint_app_lib",
    srcs = ["src/lib.rs"],
    crate_name = "slint_app",
    edition = "2021",
    deps = [
        ":build_script",  # Exposes the generated code
        "//core/state_manager",
        "@crates//:platform-dirs",
        "@crates//:slint",
        "@crates//:tokio",
        "@crates//:tracing",
        "@crates//:tracing-appender",
        "@crates//:tracing-subscriber",
    ],
)

rust_binary(
    name = "slint_app",
    srcs = ["src/main.rs"],
    edition = "2021",
    rustc_flags = select({
        "@platforms//os:windows": [
            "-Clink-args=-Wl,--start-group -lmsvcrt -lmingw32 -lmingwex -luser32 -lkernel32 -lws2_32 -lcrypt32 -ladvapi32 -Wl,--end-group",
        ],
        "//conditions:default": [],
    }),
    deps = [
        ":slint_app_lib",
        "@crates//:tokio",
    ],
)

# Android Shared Library target — compiled as cdylib for NativeActivity.
# Uses src/lib.rs (same as the desktop lib) with the full dep tree so that
# state_manager and tracing are present on Android.
rust_shared_library(
    name = "slint_app_android",
    srcs = ["src/lib.rs"],
    compile_data = [
        "src/app.slint",
    ],
    crate_name = "slint_app",
    edition = "2021",
    deps = [
        ":build_script",
        "//core/state_manager",
        "@crates//:jni",
        "@crates//:platform-dirs",
        "@crates//:slint",
        "@crates//:tokio",
        "@crates//:tracing",
        "@crates//:tracing-appender",
        "@crates//:tracing-subscriber",
    ],
)

# Transition rules for Android platforms
platform_transition_rule(
    name = "slint_app_android_x86_64",
    src = ":slint_app_android",
    platform = "//:x86_64",
)

platform_transition_rule(
    name = "slint_app_android_aarch64",
    src = ":slint_app_android",
    platform = "//:arm64-v8a",
)
