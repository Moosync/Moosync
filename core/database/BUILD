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
        "src/database.rs",
        "src/lib.rs",
        "src/migrations.rs",
        "src/test.rs",
        "src/utils.rs",
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
        "@crates//:r2d2",
        "@crates//:r2d2_sqlite",
        "@crates//:rusqlite",
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

rust_benchmark(
    name = "db_bench",
    srcs = ["benches/db_bench.rs"],
    crate_root = "benches/db_bench.rs",
    visibility = ["//visibility:public"],
    deps = [
        ":database",
        "//core/types/protos:songs_rs",
        "@crates//:uuid",
        "@crates//:criterion",
    ],
)
