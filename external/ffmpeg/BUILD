# Per-library BUILD files live under subdirectories (lame, opus, ogg, vorbis, ffmpeg).
# Aliases are defined inside each subpackage so there are no top-level targets here.
load("@rules_foreign_cc//foreign_cc:configure.bzl", "configure_make")

configure_make(
    name = "ffmpeg",
    build_data = [
        "@nasm//:nasm",
    ] + select({
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
        "--enable-pic",
        # TODO: Figure out why -logg isn't added by pkg-config
    ] + select({
        "@platforms//os:windows": [
            "--toolchain=msvc",
            "--target-os=win64",
            "--arch=x86_64",
            "--extra-cflags=-I$$EXT_BUILD_DEPS/openssl/include",
            "--extra-ldflags=-LIBPATH:$$EXT_BUILD_DEPS/openssl/lib",
            "--extra-libs=\"libssl.lib libcrypto.lib ogg.lib vorbis.lib vorbisenc.lib vorbisfile.lib crypt32.lib advapi32.lib user32.lib ws2_32.lib bcrypt.lib secur32.lib shlwapi.lib\"",
            # TODO: Hermetic pkg-config is broken, use msys2 one
            # NOTE: Only on github actions. Change it if you're building locally
            "--pkg-config=\"D:/a/_temp/msys64/usr/bin/pkg-config.exe $$BAZEL_PKG_FLAGS\"",
        ],
        "//tools:target_macos": [
            "--enable-cross-compile",
            "--target-os=darwin",
            "--arch=x86_64",
            "--cc=$$EXT_BUILD_ROOT$$/external/rules_applecross++apple_cross_toolchain+apple_cross_toolchain/wrapped_clang",
            "--cxx=$$EXT_BUILD_ROOT$$/external/rules_applecross++apple_cross_toolchain+apple_cross_toolchain/wrapped_clang_pp",
            "--ar=$$EXT_BUILD_ROOT$$/tools/darwin_ar_wrapper.sh",
            "--nm=$$EXT_BUILD_ROOT$$/external/rules_applecross++apple_cross_toolchain+apple_cross_toolchain/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/llvm-nm",
            "--ranlib=$$EXT_BUILD_ROOT$$/external/rules_applecross++apple_cross_toolchain+apple_cross_toolchain/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/llvm-ranlib",
            "--extra-libs=\"-lvorbis -lvorbisenc -lvorbisfile -logg -lm -lpthread\"",
        ],
        "//conditions:default": [
            "--extra-libs=\"-lvorbis -lvorbisenc -lvorbisfile -logg -lm -lpthread\"",
        ],
    }),
    env = select({
        "@platforms//os:windows": {
            "PATH": "$$(dirname $(execpath @nasm//:nasm))",
            # I have no idea why windows pkgconf doesn't respect PKG_CONFIG_PATH on github actions
            "BAZEL_PKG_FLAGS": " ".join([
                "--with-path=$$(cygpath -u $${EXT_BUILD_DEPS}/openssl/lib/pkgconfig)",
                "--with-path=$$(cygpath -u $${EXT_BUILD_DEPS}/ogg/lib/pkgconfig)",
                "--with-path=$$(cygpath -u $${EXT_BUILD_DEPS}/vorbis/lib/pkgconfig)",
                "--with-path=$$(cygpath -u $${EXT_BUILD_DEPS}/opus/lib/pkgconfig)",
            ]),
        },
        "//tools:target_macos": {
            "DEVELOPER_DIR": "$$EXT_BUILD_ROOT$$/external/rules_applecross++apple_cross_toolchain+apple_cross_toolchain/Xcode.app/Contents/Developer",
            "SDKROOT": "$$EXT_BUILD_ROOT$$/external/rules_applecross++apple_cross_toolchain+apple_cross_toolchain/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX15.5.sdk",
            # Cross-compile tool paths
            "AR": "$$EXT_BUILD_ROOT$$/tools/darwin_ar_wrapper.sh",
            "NM": "$$EXT_BUILD_ROOT$$/external/rules_applecross++apple_cross_toolchain+apple_cross_toolchain/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/llvm-nm",
            "RANLIB": "$$EXT_BUILD_ROOT$$/external/rules_applecross++apple_cross_toolchain+apple_cross_toolchain/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/llvm-ranlib",
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
    tool_prefix = select({
        # Escape quotes for Bazel + powershell + bash: 2^4 quotes
        "@bazel_tools//src/conditions:windows": "sed -i 's|gsub(/\\\\\\\\/,|gsub(/\\\\\\\\\\\\\\\\/,|' ffbuild/config.mak &&",
        # Fix closesocket detection: -undefined dynamic_lookup causes configure to wrongly detect closesocket
        "//tools:target_macos": "sed -i 's/HAVE_CLOSESOCKET 1/HAVE_CLOSESOCKET 0/' config.h &&",
        "//conditions:default": "",
    }),
    visibility = ["//visibility:public"],
    deps = [
        "//external/lame",
        "//external/ogg",
        "//external/openssl",
        "//external/opus",
        "//external/vorbis",
    ],
)

filegroup(
    name = "gen_dir",
    srcs = [":ffmpeg"],
    output_group = "gen_dir",
    visibility = ["//visibility:public"],
)
