# Moosync

Tauri-based music player app with Bazel build system.

## Build System

- **Bazel** for Rust compilation and cross-platform builds
- **Gradle** for Android APK packaging (calls Bazel via BuildTask.kt)
- Android targets: arm64-v8a, armeabi-v7a, x86_64

### Key Bazel targets
- `//tauri:moosync` - Desktop binary
- `//tauri:moosync_android` - Android shared library
- `//ui:moosync_ui` - Web UI bundle

### Android APK builds
Use `./gradlew assembleDebug -PskipBazel` when the native .so is already built and you just need to repackage the APK. Only omit `-PskipBazel` when the Rust code changed and needs rebuilding.

## Working Preferences

### Bazel commands
Always run bazel build/test commands in background mode (`run_in_background: true`). This allows better control over reading long build output.

### Background vs foreground commands
When running commands in background (`run_in_background: true`), do NOT pipe to `tail`, `head`, or other filters - the full output goes to a file that can be read/grepped separately. Only use pipes when running commands in foreground where you need to limit immediate output.

### Command output
When checking build output files, don't tail excessive lines (100+). Use grep to find errors first (`grep -i error`, `grep FAILED`), then tail 20-30 lines or use grep with context (`-A`/`-B` flags).

### Sequential builds
Wait for background builds to fully complete (task notification received) before starting the next one. Overlapping builds cause lock contention and confusing state.
