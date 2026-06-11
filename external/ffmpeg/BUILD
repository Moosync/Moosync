load("@rules_foreign_cc//foreign_cc:configure.bzl", "configure_make")

ANDROID_FFMPEG_ANDROID_COMMON = [
    "--target-os=android",
    "--enable-cross-compile",
    "--cc=$$EXT_BUILD_ROOT/$(CC)",
    "--cxx=$$EXT_BUILD_ROOT/$(CC)++",
    "--ld=$$EXT_BUILD_ROOT/$(CC)",
    "--ar=$$EXT_BUILD_ROOT/$(AR)",
    "--extra-libs=\"-lvorbis -lvorbisenc -lvorbisfile -logg -lopus -lssl -lcrypto -lm -lz -ldl\"",
]

ANDROID_FFMPEG_ANDROID_OPTIONS = select({
    "//:android_x86_64_config": ANDROID_FFMPEG_ANDROID_COMMON + ["--arch=x86_64"],
    "//:android_arm64_config": ANDROID_FFMPEG_ANDROID_COMMON + ["--arch=aarch64"],
    "//conditions:default": [],
})

configure_make(
    name = "ffmpeg",
    args = select({
        "@platforms//os:macos": [],
        "//conditions:default": ["-j"],
    }),
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
        "//external/openssl:msvc_compiler": [
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
        "//tools/toolchains/mingw:windows_mingw": [
            "--target-os=mingw32",
            "--arch=x86_64",
            "--enable-cross-compile",
            "--cc=$$EXT_BUILD_ROOT/$(CC)",
            "--cxx=$$EXT_BUILD_ROOT/$(CC)",
            "--ld=$$EXT_BUILD_ROOT/$(CC)",
            "--ar=$$EXT_BUILD_ROOT/$(AR)",
            "--extra-libs=\"-lssl -lcrypto -lvorbis -lvorbisenc -lvorbisfile -logg -lopus -lws2_32 -lbcrypt -lcrypt32 -lsecur32 -lshlwapi -luser32 -ladvapi32\"",
        ],
        "//:android_x86_64_config": ANDROID_FFMPEG_ANDROID_COMMON + ["--arch=x86_64"],
        "//:android_arm64_config": ANDROID_FFMPEG_ANDROID_COMMON + ["--arch=aarch64"],
        "//conditions:default": [
            "--extra-libs=\"-lvorbis -lvorbisenc -lvorbisfile -logg -lm -lpthread\"",
        ],
    }),
    env = select({
        "//external/openssl:msvc_compiler": {
            "PATH": "$$(dirname $(execpath @nasm//:nasm))",
            # I have no idea why windows pkgconf doesn't respect PKG_CONFIG_PATH on github actions
            "BAZEL_PKG_FLAGS": " ".join([
                "--with-path=$$(cygpath -u $${EXT_BUILD_DEPS}/openssl/lib/pkgconfig)",
                "--with-path=$$(cygpath -u $${EXT_BUILD_DEPS}/ogg/lib/pkgconfig)",
                "--with-path=$$(cygpath -u $${EXT_BUILD_DEPS}/vorbis/lib/pkgconfig)",
                "--with-path=$$(cygpath -u $${EXT_BUILD_DEPS}/opus/lib/pkgconfig)",
            ]),
        },
        "//conditions:default": {},
    }),
    install_prefix = "build",
    lib_source = "@ffmpeg_8_0//:all_srcs",
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
