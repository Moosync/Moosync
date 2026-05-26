load("@rules_foreign_cc//foreign_cc:cmake.bzl", "cmake")

cmake(
    name = "vorbis",
    cache_entries = select({
        "@platforms//os:windows": {
            "CMAKE_INSTALL_LIBDIR": "lib",
            "BUILD_SHARED_LIBS": "OFF",
            "OGG_ROOT": "$$EXT_BUILD_DEPS$$",
            "CMAKE_SYSTEM_NAME": "Windows",
        },
        "//conditions:default": {
            "CMAKE_INSTALL_LIBDIR": "lib",
            "BUILD_SHARED_LIBS": "OFF",
            "OGG_ROOT": "$$EXT_BUILD_DEPS$$",
        },
    }),
    lib_source = "@libvorbis//:all_srcs",
    out_static_libs = select({
        "//external/openssl:msvc_compiler": [
            "vorbis.lib",
            "vorbisenc.lib",
            "vorbisfile.lib",
        ],
        "//conditions:default": [
            "libvorbis.a",
            "libvorbisenc.a",
            "libvorbisfile.a",
        ],
    }),
    visibility = ["//visibility:public"],
    deps = ["//external/ogg"],
)
