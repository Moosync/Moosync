load("@rules_foreign_cc//foreign_cc:configure.bzl", "configure_make")

configure_make(
    name = "lame",
    configure_options = [
        "--disable-shared",
        "--enable-static",
        "--disable-frontend",  # We only need the library, not the lame CLI tool
    ],
    lib_source = "@lame//:all_srcs",
    out_static_libs = ["libmp3lame.a"],
    visibility = ["//visibility:public"],
)

# Build libopus
configure_make(
    name = "opus",
    configure_options = [
        "--disable-shared",
        "--enable-static",
        "--disable-doc",
        "--disable-extra-programs",
    ],
    lib_source = "@opus//:all_srcs",
    out_static_libs = ["libopus.a"],
    visibility = ["//visibility:public"],
)

# Build libogg
configure_make(
    name = "ogg",
    lib_source = "@libogg//:all_srcs",
    out_static_libs = ["libogg.a"],
)

# Build libvorbis
configure_make(
    name = "vorbis",
    configure_options = [
        "--disable-oggtest",
    ],
    lib_source = "@libvorbis//:all_srcs",
    out_static_libs = [
        "libvorbis.a",
        "libvorbisfile.a",
        "libvorbisenc.a",
    ],
    visibility = ["//visibility:public"],
    deps = [":ogg"],
)

configure_make(
    name = "ffmpeg",
    args = ["-j16"],
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
        "--enable-static",
    ],
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
    visibility = ["//visibility:public"],
    deps = [
        ":lame",
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
