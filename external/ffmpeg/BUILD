load("@rules_foreign_cc//foreign_cc:cmake.bzl", "cmake")
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
    configure_options = [
        "--disable-shared",
        "--enable-static",
        "--disable-frontend",
    ],
    env = select({
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

# Build libopus
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

cmake(
    name = "vorbis",
    cache_entries = {
        "CMAKE_INSTALL_LIBDIR": "lib",
        "BUILD_SHARED_LIBS": "OFF",
        "OGG_ROOT": "$$EXT_BUILD_DEPS$$",  # Critical: Helps Vorbis find Ogg
    },
    lib_source = "@libvorbis//:all_srcs",

    # 2. Define output libraries
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
    deps = [":ogg"],
)

configure_make(
    name = "ffmpeg",
    build_data = [
        "@nasm//:nasm",
    ],
    configure_in_place = True,
    configure_options = [
        "--disable-everything",
        "--disable-programs",
        "--enable-gpl",
        "--enable-version3",
        "--disable-doc",
        "--disable-htmlpages",
        "--disable-manpages",
        "--disable-shared",
        "--enable-network",
        "--enable-swresample",
        "--enable-avformat",
        "--enable-parsers",
        "--enable-protocols",
        "--enable-demuxer=aac",
        "--enable-demuxer=flac",
        "--enable-demuxer=mp3",
        "--enable-demuxer=mov",
        "--enable-demuxer=ogg",
        "--enable-demuxer=wav",
        # https://www.mail-archive.com/ffmpeg-devel@ffmpeg.org/msg33510.html
        "--enable-demuxer=w64",
        "--enable-muxer=adts",
        "--enable-muxer=flac",
        "--enable-muxer=mp3",
        "--enable-muxer=mp4",
        "--enable-muxer=ogg",
        "--enable-muxer=wav",
        "--enable-avcodec",
        "--enable-decoder=aac",
        "--enable-decoder=flac",
        "--enable-decoder=mp3",
        "--enable-decoder=vorbis",
        "--enable-decoder=opus",
        "--enable-decoder=pcm_s16le",
        "--enable-encoder=flac",
        "--enable-encoder=pcm_s16le",
        "--enable-filter=aresample",
        "--enable-filter=aformat",
        "--enable-filter=volume",
        "--enable-libmp3lame",
        "--enable-libopus",
        "--enable-libvorbis",
        "--enable-openssl",
        "--disable-vaapi",
        "--disable-vdpau",
        "--disable-libdrm",
        "--disable-ffnvcodec",
        "--enable-static",
        # TODO: Figure out why -logg isn't added by pkg-config
    ] + select({
        "@platforms//os:windows": [
            "--toolchain=msvc",
            "--target-os=win64",
            "--arch=x86_64",
            "--extra-cflags=-I$$EXT_BUILD_DEPS$$/openssl/include",
            "--extra-ldflags=-LIBPATH:$$EXT_BUILD_DEPS$$/openssl/lib",
            "--extra-libs=\"libssl.lib libcrypto.lib ogg.lib vorbis.lib vorbisenc.lib vorbisfile.lib crypt32.lib advapi32.lib user32.lib\"",
            # TODO: Hermetic pkg-config is broken, use msys2 one
            # NOTE: Only on github actions. Change it if you're building locally
            "--pkg-config=D:/a/_temp/msys64/usr/bin/pkg-config",
        ],
        "//conditions:default": [
            "--extra-libs=\"-lvorbis -lvorbisenc -lvorbisfile -logg -lm -lpthread\"",
        ],
    }),
    env = select({
        "@platforms//os:windows": {
            "PATH": "$$(dirname $(execpath @nasm//:nasm))",
            "PKG_CONFIG_PATH": ":".join([
                "$$EXT_BUILD_DEPS$$/openssl/lib/pkgconfig",
                "$$EXT_BUILD_DEPS$$/ogg/lib/pkgconfig",
                "$$EXT_BUILD_DEPS$$/vorbis/lib/pkgconfig",
                "$$EXT_BUILD_DEPS$$/opus/lib/pkgconfig",
            ]),
        },
        "//conditions:default": {},
    }),
    install_prefix = "build",
    lib_source = "@ffmpeg_7_0//:all_srcs",
    out_static_libs = [
        "libavcodec.a",
        "libavdevice.a",
        "libavfilter.a",
        "libavformat.a",
        "libavutil.a",
        "libswresample.a",
        "libswscale.a",
    ],

    # 2. Conditionally inject the fix script before make
    tool_prefix = select({
        # Escape quotes for Bazel + powershell + bash: 2^4 quotes
        "@bazel_tools//src/conditions:windows": "sed -i 's|gsub(/\\\\\\\\/,|gsub(/\\\\\\\\\\\\\\\\/,|' ffbuild/config.mak &&",
        "//conditions:default": "",
    }),
    visibility = ["//visibility:public"],
    deps = [
        ":lame",
        ":ogg",
        ":opus",
        ":vorbis",
        "//external/openssl",
    ],
)

filegroup(
    name = "gen_dir",
    srcs = [":ffmpeg"],
    output_group = "gen_dir",
    visibility = ["//visibility:public"],
)
