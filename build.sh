#!/bin/sh

set -e
set -x

script_dir=$(cd "$(dirname "$0")" && pwd)

target=$TAURI_ENV_TARGET_TRIPLE
if [ -z "$target" ]; then
    echo "Could not determine target triple from 'rustc -vV'" >&2
    exit 1
fi

is_android_build=false
if [ -n "$TAURI_ENV_TARGET_TRIPLE" ] && echo "$TAURI_ENV_TARGET_TRIPLE" | grep -q "android"; then
    is_android_build=true
fi

if [ "$is_android_build" = true ]; then
    echo "--- Android build detected ---"
    android_maker_dir="$script_dir/ffmpeg-android-maker"

    if [ ! -d "$android_maker_dir" ]; then
        echo "--- Cloning ffmpeg-android-maker ---"
        git clone https://github.com/Javernaut/ffmpeg-android-maker "$android_maker_dir"
    fi

    case "$TAURI_ENV_ARCH" in
        aarch64)
            ABI="arm64-v8a"
            TRIPLE="aarch64-linux-android"
            ;;
        armv7)
            ABI="armeabi-v7a"
            TRIPLE="armv7a-linux-androideabi"
            ;;
        i686)
            ABI="x86"
            TRIPLE="i686-linux-android"
            ;;
        x86_64)
            ABI="x86_64"
            TRIPLE="x86_64-linux-android"
            ;;
        *)
            echo "Error: Unknown architecture '$TAURI_ENV_ARCH'" >&2
            exit 1
            ;;
    esac

    android_ffmpeg_build_dir="$android_maker_dir/build/ffmpeg/$ABI"
    if [ ! -d "$android_ffmpeg_build_dir" ]; then
        echo "--- Building FFmpeg for Android ---"
        ANDROID_NDK_HOME=$NDK_HOME ANDROID_SDK_HOME=$ANDROID_HOME "$android_maker_dir/ffmpeg-android-maker.sh" --source-git-branch=release/7.0 --enable-libmp3lame --enable-libopus -android=26 -abis=$ABI
    else
        echo "--- FFmpeg for Android already built, skipping ---"
    fi

    echo "--- Copying shared libraries to jniLibs ---"
    jni_libs_dir="$script_dir/src-tauri/gen/android/app/src/main/jniLibs/$ABI"
    mkdir -p "$jni_libs_dir"
    find "$android_ffmpeg_build_dir/lib" -name "*.so" -exec cp {} "$jni_libs_dir" \;

    ffmpeg_build_dir="$android_ffmpeg_build_dir"
    ffmpeg_pkg_config_path="$android_ffmpeg_build_dir/lib/pkgconfig"
    ffmpeg_libs_dir="$android_ffmpeg_build_dir/lib"
    ffmpeg_include_dir="$android_ffmpeg_build_dir/include"
    ffmpeg_link_mode="static"
    TOOLCHAIN_ROOT="$NDK_HOME/toolchains/llvm/prebuilt/$(ls $NDK_HOME/toolchains/llvm/prebuilt | head -n 1)"
    NDK_SYSROOT="--sysroot=$TOOLCHAIN_ROOT/sysroot"
else
    echo "--- Non-Android build detected ---"
    ffmpeg_dir="$script_dir/ffmpeg"
    ffmpeg_build_dir="$ffmpeg_dir/build-ffmpeg-$target"
    prefix="$ffmpeg_build_dir/build"

    extra_configure_flags=()
    if echo "$target" | grep -q "apple-darwin"; then
        extra_configure_flags+=("--extra-cflags=-I/opt/homebrew/include")
        extra_configure_flags+=("--extra-ldflags=-L/opt/homebrew/lib")
    elif echo "$target" | grep -q "windows"; then
        vcpkg_triplet=""
        case "$TAURI_ENV_ARCH" in
            x86_64) vcpkg_triplet="x64-windows" ;;
            i686) vcpkg_triplet="x86-windows" ;;
        esac
        if [ -n "$VCPKG_ROOT" ] && [ -n "$vcpkg_triplet" ]; then
            POSIX_VCPKG_ROOT=$(echo "$VCPKG_ROOT" | sed 's/\\/\//g' | sed 's/://')
            if echo "$VCPKG_ROOT" | grep -q '^\w:'; then
                 # For drive letters (e.g., C:), prefix with a slash
                POSIX_VCPKG_ROOT="/$POSIX_VCPKG_ROOT"
            fi
            VCPKG_INCLUDE_DIR="$POSIX_VCPKG_ROOT/installed/$vcpkg_triplet/include"
            VCPKG_LIB_DIR="$POSIX_VCPKG_ROOT/installed/$vcpkg_triplet/lib"

            extra_cflags="-I$VCPKG_INCLUDE_DIR -I$POSIX_VCPKG_ROOT/installed/$vcpkg_triplet-static-md/include"

            extra_ldflags="-L$VCPKG_LIB_DIR -L$POSIX_VCPKG_ROOT/packages/openssl_$vcpkg_triplet-static-md/lib"

            extra_configure_flags+=("--extra-cflags=$extra_cflags") 
            extra_configure_flags+=("--extra-ldflags=$extra_ldflags")
            extra_configure_flags+=("--toolchain=msvc")
        fi
    fi

    if [ ! -d "$ffmpeg_dir" ]; then
        echo "--- Cloning FFmpeg ---"
        git clone https://github.com/ffmpeg/ffmpeg --depth 1 --single-branch --branch release/7.0 "$ffmpeg_dir"
    fi

    if [ ! -d "$ffmpeg_build_dir" ]; then
        echo "--- Building FFmpeg ---"
        mkdir -p "$ffmpeg_build_dir"
        
        (
            cd "$ffmpeg_build_dir" || exit

            libvorbis_flag="--enable-libvorbis"
            if echo "$target" | grep -q "windows"; then
                libvorbis_flag=""
            fi

            echo "--- Configuring FFmpeg ---"
            ../configure \
                "${extra_configure_flags[@]}" \
                --prefix="$prefix" \
                --disable-everything \
                --disable-programs \
                --enable-gpl \
                --enable-version3 \
                --disable-doc \
                --disable-htmlpages \
                --disable-manpages \
                --disable-shared \
                --enable-network \
                --enable-swresample \
                --enable-avformat \
                --enable-parsers \
                --enable-protocols \
                --enable-demuxer=aac \
                --enable-demuxer=flac \
                --enable-demuxer=mp3 \
                --enable-demuxer=mov \
                --enable-demuxer=ogg \
                --enable-demuxer=wav \
                --enable-muxer=adts \
                --enable-muxer=flac \
                --enable-muxer=mp3 \
                --enable-muxer=mp4 \
                --enable-muxer=ogg \
                --enable-muxer=wav \
                --enable-avcodec \
                --enable-decoder=aac \
                --enable-decoder=flac \
                --enable-decoder=mp3 \
                --enable-decoder=vorbis \
                --enable-decoder=opus \
                --enable-decoder=pcm_s16le \
                --enable-encoder=flac \
                --enable-encoder=pcm_s16le \
                --enable-filter=aresample \
                --enable-filter=aformat \
                --enable-filter=volume \
                --enable-libmp3lame \
                --enable-libopus \
                $libvorbis_flag \
                --enable-openssl \
                --disable-vaapi \
                --disable-vdpau \
                --disable-libdrm \
                --enable-static

            echo "--- Compiling FFmpeg ---"
            make -j"$(nproc)"

            echo "--- Installing FFmpeg ---"
            make install
        )
    fi

    ffmpeg_pkg_config_path="$ffmpeg_build_dir/build/lib/pkgconfig"
    ffmpeg_libs_dir="$ffmpeg_build_dir/build/lib"
    ffmpeg_include_dir="$ffmpeg_build_dir/build/include"
    ffmpeg_link_mode="static"
fi

# Function to update .cargo/config.toml in a cross-platform way
update_cargo_config() {
    expression="$1"
    if echo "$target" | grep -q "apple-darwin"; then
        sed -i '' -e "$expression" .cargo/config.toml
    else
        sed -i -e "$expression" .cargo/config.toml
    fi
}

# Update .cargo/config.toml with ffmpeg paths
update_cargo_config "s|FFMPEG_PKG_CONFIG_PATH = \".*\"|FFMPEG_PKG_CONFIG_PATH = \"$ffmpeg_pkg_config_path\"|"
update_cargo_config "s|FFMPEG_INCLUDE_DIR = \".*\"|FFMPEG_INCLUDE_DIR = \"$ffmpeg_include_dir\"|"
update_cargo_config "s|FFMPEG_LIBS_DIR = \".*\"|FFMPEG_LIBS_DIR = \"$ffmpeg_libs_dir\"|"
update_cargo_config "s|FFMPEG_LINK_MODE = \".*\"|FFMPEG_LINK_MODE = \"$ffmpeg_link_mode\"|"
update_cargo_config "s|BINDGEN_EXTRA_CLANG_ARGS = \".*\"|BINDGEN_EXTRA_CLANG_ARGS = \"$NDK_SYSROOT\"|"

# delete recusrive symlink
rm -rf "$ffmpeg_build_dir/src"

echo "Updated ffmpeg config in .cargo/config.toml"flac