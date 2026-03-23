load("@rules_foreign_cc//foreign_cc:cmake.bzl", "cmake")

cmake(
    name = "opus",
    build_data = select({
        "//tools:target_macos": [
            "@apple_cross_toolchain//:toolchain_files",
            "@apple_cross_toolchain//:wrapped_clang",
            "@apple_cross_toolchain//:wrapped_clang_pp",
            "@apple_cross_toolchain//:cc_wrapper.sh",
            "@apple_cross_toolchain//:xcrunwrapper.sh",
        ],
        "//conditions:default": [],
    }),
    cache_entries = {
        "CMAKE_INSTALL_LIBDIR": "lib",
        "CMAKE_POSITION_INDEPENDENT_CODE": "ON",
        "BUILD_SHARED_LIBS": "OFF",
        "BUILD_TESTING": "OFF",
        "OPUS_BUILD_PROGRAMS": "OFF",
    } | select({
        "//tools:target_macos": {
            "CMAKE_SYSTEM_NAME": "Darwin",
            "CMAKE_CROSSCOMPILING": "ON",
            "CMAKE_OSX_ARCHITECTURES": "x86_64",
        },
        "//conditions:default": {},
    }),
    env = select({
        "//tools:target_macos": {
            "DEVELOPER_DIR": "$$EXT_BUILD_ROOT$$/external/rules_applecross++apple_cross_toolchain+apple_cross_toolchain/Xcode.app/Contents/Developer",
            "SDKROOT": "$$EXT_BUILD_ROOT$$/external/rules_applecross++apple_cross_toolchain+apple_cross_toolchain/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX15.5.sdk",
        },
        "//conditions:default": {},
    }),
    lib_source = "@opus//:all_srcs",
    out_static_libs = select({
        "@platforms//os:windows": ["opus.lib"],
        "//conditions:default": ["libopus.a"],
    }),
    visibility = ["//visibility:public"],
)
