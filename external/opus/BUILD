load("@rules_foreign_cc//foreign_cc:cmake.bzl", "cmake")

cmake(
    name = "opus",
    cache_entries = select({
        "@platforms//os:windows": {
            "CMAKE_INSTALL_LIBDIR": "lib",
            "BUILD_SHARED_LIBS": "OFF",
            "BUILD_TESTING": "OFF",
            "OPUS_BUILD_PROGRAMS": "OFF",
            "CMAKE_SYSTEM_NAME": "Windows",
        },
        "//conditions:default": {
            "CMAKE_INSTALL_LIBDIR": "lib",
            "BUILD_SHARED_LIBS": "OFF",
            "BUILD_TESTING": "OFF",
            "OPUS_BUILD_PROGRAMS": "OFF",
        },
    }),
    lib_source = "@opus//:all_srcs",
    out_static_libs = select({
        "//external/openssl:msvc_compiler": ["opus.lib"],
        "//conditions:default": ["libopus.a"],
    }),
    visibility = ["//visibility:public"],
)
