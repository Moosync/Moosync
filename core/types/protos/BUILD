load("@rules_proto//proto:defs.bzl", "proto_library")
load("@rules_rust_prost//:defs.bzl", "rust_prost_library")

package(default_visibility = ["//visibility:public"])

proto_library(
    name = "extensions_proto",
    srcs = ["extensions.proto"],
    deps = [
        "//core/types/protos:songs_proto",
        "//core/types/protos/ui:player_proto",
        "//core/types/protos/ui:preferences_proto",
        "@com_google_protobuf//:struct_proto",
    ],
)

rust_prost_library(
    name = "extensions_rs",
    proto = ":extensions_proto",
    visibility = ["//visibility:public"],
)

proto_library(
    name = "songs_proto",
    srcs = ["songs.proto"],
)

rust_prost_library(
    name = "songs_rs",
    proto = ":songs_proto",
    visibility = ["//visibility:public"],
)

proto_library(
    name = "themes_proto",
    srcs = ["themes.proto"],
    deps = ["//core/types/protos:songs_proto"],
)

rust_prost_library(
    name = "themes_rs",
    proto = ":themes_proto",
    visibility = ["//visibility:public"],
)
