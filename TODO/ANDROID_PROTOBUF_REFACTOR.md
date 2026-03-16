# Android Kotlin Protobuf Refactor

The Kotlin code in Android plugins predates the protobuf refactor and uses manually defined data classes that don't match the protobuf schema. This causes deserialization failures when Rust tries to parse Kotlin-serialized data.

## Problem

**Kotlin Song class** (`lib/tauri-plugin-file-scanner/android/src/main/java/utils/Song.kt`):
```kotlin
data class Song(
    val title: String,
    val duration: Long,
    val path: String?,
    val artist: List<Artist>?,  // singular field name
    val album: Album?,
    ...
)
```

**Protobuf Song** (`core/types/protos/songs.proto`):
```protobuf
message Song {
  InnerSong song = 1;        // nested structure
  optional Album album = 2;
  repeated Artist artists = 3;  // plural field name
  repeated Genre genre = 4;
}
```

When `AudioScanner` serializes songs with Gson and sends to Rust, `serde_json::from_str` fails because the structure doesn't match. Songs are never inserted into the database.

## Affected Files

- `lib/tauri-plugin-file-scanner/android/src/main/java/utils/Song.kt`
- `lib/tauri-plugin-file-scanner/android/src/main/java/AudioScanner.kt`
- `lib/tauri-plugin-file-scanner/src/mobile.rs`
- `lib/tauri-plugin-audioplayer/android/` (likely similar issues)

## Solution

1. **Add protobuf plugin to Android build**
   - Add `com.google.protobuf` Gradle plugin
   - Configure protobuf generation for Kotlin

2. **Generate Kotlin classes from protos**
   - Generate from `core/types/protos/songs.proto`
   - Possibly other protos used in IPC

3. **Update AudioScanner**
   - Use generated `Song`, `InnerSong`, `Artist`, `Album`, `Genre` classes
   - Remove manual data classes in `utils/`

4. **Update serialization**
   - Use protobuf-JSON serialization (`JsonFormat`) for compatibility with Rust's prost-serde
   - Or use binary protobuf if preferred

5. **Update Rust deserialization**
   - Ensure `mobile.rs` correctly deserializes protobuf-JSON

## Quick Fix (Not Recommended)

Manually restructure Kotlin Song class to match proto schema without protobuf codegen:
```kotlin
data class InnerSong(val title: String?, val duration: Double?, ...)
data class Song(val song: InnerSong, val album: Album?, val artists: List<Artist>?, val genre: List<Genre>?)
```

This is fragile and will drift out of sync with proto changes.

## Related

This likely affects `tauri-plugin-audioplayer` as well, which has similar Kotlin data classes for media playback state.
