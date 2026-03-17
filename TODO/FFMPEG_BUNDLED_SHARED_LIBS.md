# FFmpeg Bundled Shared Libraries

Switch FFmpeg from static linking (with PIC) to bundled shared libraries to eliminate PIC overhead and reduce CPU usage during audio decode.

## Current State

FFmpeg and codec dependencies (lame, opus, vorbis, ogg, openssl) are built as static libraries with `-fPIC` / `--enable-pic` flags. This is required because Rust/Fedora build PIE executables by default.

PIC overhead: ~1-3% CPU on decode-heavy workloads due to:
- One register reserved for GOT pointer
- Extra indirection on global data access

## Proposed Solution

Build FFmpeg as a shared library and bundle it with the app distribution.

### 1. Bazel FFmpeg build changes

```python
# external/ffmpeg/BUILD
configure_options = [
    "--disable-static",
    "--enable-shared",
    # ... rest unchanged
]

out_shared_libs = [
    "libavcodec.so",
    "libavformat.so",
    "libavutil.so",
    "libswresample.so",
    # etc.
]
```

### 2. Codec dependencies strategy

Build lame, opus, vorbis, ogg statically *into* the FFmpeg shared lib (simpler than multiple .so files):
- Keep codec libs as static with PIC
- FFmpeg links them into its .so output
- Result: single libav*.so set with all codecs embedded

### 3. RPATH configuration

```python
# In Bazel rust_binary:
rustc_flags = ["-C", "link-arg=-Wl,-rpath,$ORIGIN/lib"]
```

### 4. Platform-specific packaging

| Platform | Library location | Mechanism |
|----------|------------------|-----------|
| Linux | `lib/` next to binary | RPATH `$ORIGIN/lib` |
| macOS | `Frameworks/` in .app | `@loader_path/../Frameworks/` |
| Windows | Same dir as .exe | DLL search path (automatic) |
| Android | `jniLibs/` in APK | Already dynamic |

### 5. Tauri bundling

Update `tauri.conf.json` to include .so files as resources, ensure correct placement in each bundle type (deb, AppImage, dmg, etc.).

## Effort Estimate

| Task | Time |
|------|------|
| Bazel build changes | 2-3 hours |
| RPATH setup | 1-2 hours |
| Tauri Linux packaging | 3-4 hours |
| Cross-platform (macOS/Windows) | 4-6 hours |
| Testing & edge cases | 2-3 hours |
| **Total** | **2-3 days** |

## When to prioritize

- FFmpeg shows up in CPU profiles as significant
- Want independent FFmpeg updates without full rebuild
- Doing packaging work anyway

## References

- Original issue: PIC linker errors when building without `--enable-pic`
- FFmpeg configure docs: https://ffmpeg.org/ffmpeg-all.html
