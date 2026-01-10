load("@rules_rust//cargo:defs.bzl", "cargo_build_script")
load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

filegroup(
    name = "migrations",
    srcs = glob([
        "migrations/**/*",
        "migrations_cache/**/*",
    ]),
)

rust_library(
    name = "database",
    srcs = [
        "src/cache.rs",
        "src/cache_schema.rs",
        "src/database.rs",
        "src/lib.rs",
        "src/macros.rs",
        "src/migrations.rs",
        "src/models.rs",
        "src/schema.rs",
        "src/test.rs",
    ],
    compile_data = [
        ":migrations",
    ],
    crate_root = "src/lib.rs",
    visibility = ["//visibility:public"],
    deps = [
        ":build_script",
        "//core/types",
        "//core/types/protos:songs_rs",
        "@crates//:diesel",
        "@crates//:diesel_logger",
        "@crates//:diesel_migrations",
        "@crates//:libsqlite3-sys",
        "@crates//:serde",
        "@crates//:serde_json",
        "@crates//:tracing",
        "@crates//:uuid",
    ],
)

cargo_build_script(
    name = "build_script",
    srcs = ["build.rs"],
    crate_root = "build.rs",
    visibility = ["//visibility:public"],
    deps = ["@crates//:glob"],
)

rust_test(
    name = "database_test",
    crate = ":database",
)
