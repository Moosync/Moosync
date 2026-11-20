load("@rules_rust//cargo:defs.bzl", "cargo_build_script")
load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")
load("//:defs.bzl", "rust_benchmark")

config_setting(
    name = "sqlx_live",
    values = {"define": "sqlx_offline=false"},
)

filegroup(
    name = "migrations",
    srcs = glob([
        "migrations/**/*",
    ]),
)

filegroup(
    name = "sqlx_offline_cache",
    srcs = glob(
        [".sqlx/**/*"],
        allow_empty = True,
    ),
)

rust_library(
    name = "database",
    srcs = [
        "src/analytics.rs",
        "src/cache.rs",
        "src/cache_test.rs",
        "src/database.rs",
        "src/database_test.rs",
        "src/entities.rs",
        "src/error.rs",
        "src/lib.rs",
        "src/lib_test_smoke.rs",
        "src/migrations.rs",
        "src/migrations_test.rs",
        "src/models.rs",
        "src/playlists.rs",
        "src/pool.rs",
        "src/search.rs",
        "src/songs.rs",
        "src/utils.rs",
        "src/utils_test.rs",
    ],
    compile_data = [
        ":migrations",
        ":sqlx_offline_cache",
    ],
    crate_root = "src/lib.rs",
    proc_macro_deps = [
        "//core/plugin_macro",
    ],
    rustc_env = select({
        ":sqlx_live": {"SQLX_OFFLINE": "false"},
        "//conditions:default": {"SQLX_OFFLINE": "true"},
    }),
    visibility = ["//visibility:public"],
    deps = [
        ":build_script",
        "//core/types",
        "//core/types/protos:songs_rs",
        "@crates//:prost",
        "@crates//:sqlx",
        "@crates//:thiserror",
        "@crates//:tokio",
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
    deps = [
        "@crates//:assertables",
        "@crates//:rstest",
        "@crates//:tempdir",
        "@crates//:tracing-test",
    ],
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
        "@crates//:tokio",
        "@crates//:uuid",
    ],
)
