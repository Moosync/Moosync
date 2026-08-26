load("@rules_rust//cargo:defs.bzl", "cargo_build_script")
load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")
load("//:defs.bzl", "rust_benchmark")

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
        "src/cache_test.rs",
        "src/database.rs",
        "src/database_test.rs",
        "src/error.rs",
        "src/lib.rs",
        "src/lib_test_smoke.rs",
        "src/migrations.rs",
        "src/migrations_test.rs",
        "src/utils.rs",
        "src/utils_test.rs",
    ],
    compile_data = [
        ":migrations",
    ],
    crate_root = "src/lib.rs",
    proc_macro_deps = [
        "//core/plugin_macro",
    ],
    visibility = ["//visibility:public"],
    deps = [
        ":build_script",
        "//core/types",
        "//core/types/protos:songs_rs",
        "@crates//:libsqlite3-sys",
        "@crates//:r2d2",
        "@crates//:r2d2_sqlite",
        "@crates//:rusqlite",
        "@crates//:serde",
        "@crates//:serde_json",
        "@crates//:tempdir",
        "@crates//:thiserror",
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

rust_benchmark(
    name = "db_bench",
    srcs = ["benches/db_bench.rs"],
    crate_root = "benches/db_bench.rs",
    visibility = ["//visibility:public"],
    deps = [
        ":database",
        "//core/types/protos:songs_rs",
        "@crates//:criterion",
        "@crates//:uuid",
    ],
)
