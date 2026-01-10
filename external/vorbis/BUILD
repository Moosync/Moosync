load("@rules_foreign_cc//foreign_cc:cmake.bzl", "cmake")

cmake(
    name = "vorbis",
    cache_entries = {
        "CMAKE_INSTALL_LIBDIR": "lib",
        "BUILD_SHARED_LIBS": "OFF",
        "OGG_ROOT": "$$EXT_BUILD_DEPS$$",
    },
    lib_source = "@libvorbis//:all_srcs",
    out_static_libs = select({
        "@platforms//os:windows": [
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
