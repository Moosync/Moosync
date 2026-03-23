load("@rules_foreign_cc//foreign_cc:configure.bzl", "configure_make")
load("@rules_foreign_cc//foreign_cc:make.bzl", "make_variant")

make_variant(
    name = "lame_msvc",
    # We can pass the makefile flag directly here
    args = [
        "/f Makefile.MSVC",
        "comp=msvc",
        "ASM=nasm",
        "MACHINE=/MACHINE:X64",
        "MSVCVER=Win64",
    ],
    build_data = [
        "@nasm//:nasm",
    ],
    env = {
        "PATH": "$$(dirname $(execpath @nasm//:nasm))",
    },
    lib_source = "@lame//:all_srcs",
    # Windows ffmpeg tries to find mp3lame.lib not libmp3lame.lib
    out_static_libs = ["mp3lame.lib"],
    # Manual install still required as NMake files don't support 'install'
    postfix_script = "\n".join([
        "mkdir -p $$INSTALLDIR/lib",
        "mkdir -p $$INSTALLDIR/include/lame",
        "find . -name 'libmp3lame*.lib' -exec cp {} $$INSTALLDIR/lib/mp3lame.lib \\; -quit",
        "cp include/lame.h $$INSTALLDIR/include/lame/",
    ]),
    tags = ["manual"],
    toolchain = "@rules_foreign_cc//toolchains:preinstalled_nmake_toolchain",
)

configure_make(
    name = "lame_nix",
    build_data = select({
        "//tools:target_macos": [
            "@apple_cross_toolchain//:toolchain_files",
            "@apple_cross_toolchain//:wrapped_clang",
            "@apple_cross_toolchain//:wrapped_clang_pp",
            "@apple_cross_toolchain//:cc_wrapper.sh",
            "@apple_cross_toolchain//:xcrunwrapper.sh",
            "//tools:darwin_ar_wrapper.sh",
        ],
        "//conditions:default": [],
    }),
    configure_options = [
        "--disable-shared",
        "--enable-static",
        "--disable-frontend",
        "--with-pic",
    ] + select({
        "//tools:target_macos": ["--host=x86_64-apple-darwin"],
        "//conditions:default": [],
    }),
    env = select({
        "//tools:target_macos": {
            "DEVELOPER_DIR": "$$EXT_BUILD_ROOT$$/external/rules_applecross++apple_cross_toolchain+apple_cross_toolchain/Xcode.app/Contents/Developer",
            "SDKROOT": "$$EXT_BUILD_ROOT$$/external/rules_applecross++apple_cross_toolchain+apple_cross_toolchain/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX15.5.sdk",
            # Cross-compile tool paths - use wrapper that translates GNU ar syntax to llvm-libtool-darwin
            "AR": "$$EXT_BUILD_ROOT$$/tools/darwin_ar_wrapper.sh",
            "NM": "$$EXT_BUILD_ROOT$$/external/rules_applecross++apple_cross_toolchain+apple_cross_toolchain/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/llvm-nm",
            "RANLIB": "$$EXT_BUILD_ROOT$$/external/rules_applecross++apple_cross_toolchain+apple_cross_toolchain/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/llvm-ranlib",
            "STRIP": "$$EXT_BUILD_ROOT$$/external/rules_applecross++apple_cross_toolchain+apple_cross_toolchain/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/llvm-strip",
        },
        "@platforms//os:macos": {
            "AR": "",
        },
        "//conditions:default": {},
    }),
    lib_name = "lame",
    lib_source = "@lame//:all_srcs",
    out_static_libs = ["libmp3lame.a"],
    tags = ["manual"],
)

alias(
    name = "lame",
    actual = select({
        "@platforms//os:windows": ":lame_msvc",
        "//conditions:default": ":lame_nix",
    }),
    visibility = ["//visibility:public"],
)
