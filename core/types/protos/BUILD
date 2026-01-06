load("@rules_go//proto:def.bzl", "go_proto_library")
load("@rules_proto//proto:defs.bzl", "proto_library")
load("@rules_rust_prost//:defs.bzl", "rust_prost_library")

package(default_visibility = ["//visibility:public"])

proto_library(
    name = "extensions_proto",
    srcs = ["extensions.proto"],
    deps = [
        ":songs_proto",
        ":ui_proto",
        "@com_google_protobuf//:empty_proto",
        "@com_google_protobuf//:struct_proto",
    ],
)

proto_library(
    name = "ui_proto",
    srcs = ["ui.proto"],
    deps = [
        "@com_google_protobuf//:struct_proto",
    ],
)

rust_prost_library(
    name = "ui_rs",
    proto = ":ui_proto",
    visibility = ["//visibility:public"],
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
    deps = [":songs_proto"],
)

rust_prost_library(
    name = "themes_rs",
    proto = ":themes_proto",
    visibility = ["//visibility:public"],
)

go_proto_library(
    name = "ui_go_proto",
    importpath = "github.com/moosync/moosync/types/ui",
    protos = [":ui_proto"],
    visibility = ["//visibility:public"],
)

go_proto_library(
    name = "songs_go_proto",
    importpath = "github.com/moosync/moosync/types/songs",
    protos = [":songs_proto"],
    visibility = ["//visibility:public"],
)

go_proto_library(
    name = "extensions_go_proto",
    importpath = "github.com/moosync/moosync/types/extensions",
    protos = [":extensions_proto"],
    visibility = ["//visibility:public"],
    deps = [
        ":songs_go_proto",
        ":ui_go_proto",
    ],
)

go_proto_library(
    name = "themes_go_proto",
    importpath = "github.com/moosync/moosync/types/themes",
    protos = [":themes_proto"],
    visibility = ["//visibility:public"],
    deps = [":songs_go_proto"],
)
