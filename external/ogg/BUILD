load("@rules_foreign_cc//foreign_cc:cmake.bzl", "cmake")

cmake(
    name = "ogg",
    cache_entries = select({
        "@platforms//os:windows": {
            "CMAKE_INSTALL_LIBDIR": "lib",
            "BUILD_SHARED_LIBS": "OFF",
            "BUILD_TESTING": "OFF",
            "CMAKE_SYSTEM_NAME": "Windows",
        },
        "//conditions:default": {
            "CMAKE_INSTALL_LIBDIR": "lib",
            "BUILD_SHARED_LIBS": "OFF",
            "BUILD_TESTING": "OFF",
        },
    }),
    lib_source = "@libogg//:all_srcs",
    out_static_libs = select({
        "//external/openssl:msvc_compiler": ["ogg.lib"],
        "//conditions:default": ["libogg.a"],
    }),
    visibility = ["//visibility:public"],
)
