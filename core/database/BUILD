load("@rules_rust//cargo:defs.bzl", "cargo_build_script")
load("@rules_rust//rust:defs.bzl", "rust_shared_library", "rust_test")

# gazelle:rust_cargo_lockfile Cargo.lock
# gazelle:rust_crates_prefix @crates//:
# gazelle:rust_mode generate_from_cargo

filegroup(
    name = "migrations",
    srcs = glob([
        "migrations/**/*",
        "migrations_cache/**/*",
    ]),
)

rust_shared_library(
    name = "database",
    srcs = [
        "src/cache.rs",
        "src/cache_schema.rs",
        "src/database.rs",
        "src/lib.rs",
        "src/macros.rs",
        "src/migrations.rs",
        "src/schema.rs",
        "src/test.rs",
        "src/models.rs",
    ],
    compile_data = [
        "Cargo.toml",
        ":migrations",
    ],
    visibility = ["//visibility:public"],
    deps = [
        ":build_script",
        "//core/types",
        "@crates//:diesel",
        "@crates//:diesel_logger",
        "@crates//:diesel_migrations",
        "@crates//:serde",
        "@crates//:serde_json",
        "@crates//:tracing",
        "@crates//:uuid",
    ],
)

cargo_build_script(
    name = "build_script",
    srcs = ["build.rs"],
    compile_data = ["Cargo.toml"],
    crate_root = "build.rs",
    visibility = ["//visibility:public"],
    deps = ["@crates//:glob"],
)

rust_test(
    name = "database_test",
    compile_data = ["Cargo.toml"],
    crate = ":database",
)
