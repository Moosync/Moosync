load("@rules_foreign_cc//foreign_cc:cmake.bzl", "cmake")

cmake(
    name = "ogg",
    cache_entries = {
        "CMAKE_INSTALL_LIBDIR": "lib",
        "BUILD_SHARED_LIBS": "OFF",
        "BUILD_TESTING": "OFF",
    },
    lib_source = "@libogg//:all_srcs",
    out_static_libs = select({
        "@platforms//os:windows": ["ogg.lib"],
        "//conditions:default": ["libogg.a"],
    }),
    visibility = ["//visibility:public"],
)
