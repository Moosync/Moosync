load("@extensions_sdk//:defs.bzl", "PROVIDER_SCOPES", "rust_extension")

package(default_visibility = ["//visibility:public"])

rust_extension(
    name = "sample_extension",
    package_name = "rs.sample",
    srcs = ["src/lib.rs"],
    allowed_hosts = [
        "api.example.com",
        "*.moosync.app",
    ],
    allowed_paths = {
        "/tmp/moosync": "/tmp/sandbox",
        "/music": "/media/music",
    },
    display_name = "Sample Extension (Rust)",
    scopes = [
        PROVIDER_SCOPES.SEARCH,
        PROVIDER_SCOPES.PLAYBACK_DETAILS,
    ],
    version = "1.0.0",
)

alias(
    name = "default",
    actual = ":sample_extension",
)
