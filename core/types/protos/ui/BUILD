load("@rules_proto//proto:defs.bzl", "proto_library")
load("@rules_rust_prost//:defs.bzl", "rust_prost_library")

package(default_visibility = ["//visibility:public"])

proto_library(
    name = "player_proto",
    srcs = ["player.proto"],
)

rust_prost_library(
    name = "player_rs",
    proto = ":player_proto",
    visibility = ["//visibility:public"],
)

proto_library(
    name = "preferences_proto",
    srcs = ["preferences.proto"],
    deps = [
        "//core/types/protos:entities_proto",
        "//core/types/protos:songs_proto",
        "@com_google_protobuf//:struct_proto",
    ],
)

rust_prost_library(
    name = "preferences_rs",
    proto = ":preferences_proto",
    visibility = ["//visibility:public"],
)
