load("@rules_foreign_cc//foreign_cc:cmake.bzl", "cmake")

cmake(
    name = "opus",
    cache_entries = {
        "CMAKE_INSTALL_LIBDIR": "lib",
        "BUILD_SHARED_LIBS": "OFF",
        "BUILD_TESTING": "OFF",
        "OPUS_BUILD_PROGRAMS": "OFF",
    },
    lib_source = "@opus//:all_srcs",
    out_static_libs = select({
        "@platforms//os:windows": ["opus.lib"],
        "//conditions:default": ["libopus.a"],
    }),
    visibility = ["//visibility:public"],
)
