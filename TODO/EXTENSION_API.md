# Extension API

This document outlines the API for developing extensions for Moosync. Extensions in Moosync are built using WebAssembly (WASM) and can be written in any language that compiles to WASM.

## Extension structure

Extensions should follow this directory structure:

```
extension-name/
├── manifest.json          # Extension manifest file
├── main.wasm              # Main WASM binary
├── icon.png               # Extension icon (optional)
├── LICENSE                # Extension license (optional)
└── README.md              # Extension documentation (optional)
```

### Directory Guidelines:
- Use kebab-case for extension directory names
- Keep the structure minimal and organized
- Include all necessary files for the extension to function
- Provide clear documentation in README.md

## Extension manifest

The `manifest.json` file describes your extension and its capabilities:

```json
{
  "moosyncExtension": true,
  "displayName": "My Awesome Extension",
  "name": "my-awesome-extension",
  "version": "1.0.0",
  "author": "Your Name",
  "extensionEntry": "main.wasm",
  "icon": "icon.png",
  "permissions": {
    "scopes": ["Songs", "Playlists", "Preferences"],
    "hosts": ["api.example.com", "cdn.example.com"]
  }
}
```

### Manifest Fields:
- **moosyncExtension** (boolean, required): Must be `true` to identify as a Moosync extension
- **displayName** (string, required): Human-readable name shown in UI
- **name** (string, required): Unique identifier using kebab-case
- **version** (string, required): Semantic version (e.g., "1.0.0")
- **author** (string, optional): Extension author name
- **extensionEntry** (string, required): Path to main WASM file
- **icon** (string, optional): Path to extension icon
- **permissions** (object, optional): Required permissions for the extension

### Permission Scopes:
- **Songs**: Access to song data and operations
- **Playlists**: Access to playlist data and operations  
- **Preferences**: Access to user preferences
- **Network**: Network access for external API calls
- **FileSystem**: Limited file system access
- **Player**: Player state and controls

### Host Permissions:
List of allowed external hosts for network requests.

## WASM exports

Extensions must export specific functions that Moosync can call:

### Required Exports:

```rust
// Extension lifecycle
#[no_mangle]
pub extern "C" fn on_started() {
    // Called when extension is loaded and started
}

#[no_mangle] 
pub extern "C" fn on_stopped() {
    // Called when extension is stopped or unloaded
}

// Event handlers
#[no_mangle]
pub extern "C" fn on_song_changed(song_data: *const u8, len: usize) {
    // Called when current song changes
}

#[no_mangle]
pub extern "C" fn on_player_state_changed(state: u32) {
    // Called when player state changes (play/pause/stop)
}

#[no_mangle]
pub extern "C" fn on_queue_changed(queue_data: *const u8, len: usize) {
    // Called when queue is modified
}
```

### Optional Exports:

```rust
// Provider capabilities
#[no_mangle]
pub extern "C" fn get_provider_scopes() -> *const u8 {
    // Return supported provider scopes (search, playlists, etc.)
}

#[no_mangle]
pub extern "C" fn search_songs(query: *const u8, len: usize) -> *const u8 {
    // Search for songs from external sources
}

#[no_mangle]
pub extern "C" fn get_playlists(user_id: *const u8, len: usize) -> *const u8 {
    // Get playlists from external provider
}

#[no_mangle]
pub extern "C" fn get_playlist_songs(playlist_id: *const u8, len: usize) -> *const u8 {
    // Get songs from a specific playlist
}

// Context menu extensions
#[no_mangle]
pub extern "C" fn get_context_menu_items(context: *const u8, len: usize) -> *const u8 {
    // Return custom context menu items
}

#[no_mangle]
pub extern "C" fn on_context_menu_clicked(item_id: *const u8, len: usize) {
    // Handle context menu item clicks
}

// Custom UI elements
#[no_mangle]
pub extern "C" fn get_custom_ui() -> *const u8 {
    // Return custom UI element descriptions
}
```

### Host Functions (Available to Extensions):

Extensions can call these host functions provided by Moosync:

```rust
// Song operations
extern "C" {
    fn get_songs(options: *const u8, len: usize) -> *const u8;
    fn add_songs(songs: *const u8, len: usize) -> bool;
    fn update_song(song: *const u8, len: usize) -> bool;
    fn remove_song(song: *const u8, len: usize) -> bool;
}

// Playlist operations  
extern "C" {
    fn get_playlists() -> *const u8;
    fn add_playlist(playlist: *const u8, len: usize) -> bool;
    fn add_to_playlist(request: *const u8, len: usize) -> bool;
}

// Player operations
extern "C" {
    fn get_current_song() -> *const u8;
    fn get_player_state() -> u32;
    fn get_volume() -> f64;
    fn get_time() -> f64;
    fn get_queue() -> *const u8;
}

// Preferences
extern "C" {
    fn get_preference(key: *const u8, len: usize) -> *const u8;
    fn set_preference(data: *const u8, len: usize) -> bool;
    fn get_secure(key: *const u8, len: usize) -> *const u8;
    fn set_secure(data: *const u8, len: usize) -> bool;
}

// Utility functions
extern "C" {
    fn open_external_url(url: *const u8, len: usize);
    fn register_oauth(provider: *const u8, len: usize);
    fn log_message(level: u32, message: *const u8, len: usize);
}
```

### Data Serialization:

All data passed between Moosync and extensions is serialized as JSON. Extensions should:

1. Deserialize input parameters from JSON
2. Serialize return values to JSON
3. Handle serialization errors gracefully
4. Use the provided type definitions from the Moosync SDK

### Error Handling:

Extensions should handle errors gracefully:

```rust
// Return error objects for failed operations
{
    "success": false,
    "error": "Error message",
    "error_code": "ERROR_CODE"
}

// Return success objects for successful operations
{
    "success": true,
    "data": { /* result data */ }
}
```

### Development Guidelines:

1. **Performance**: Extensions run in the main thread, avoid blocking operations
2. **Memory**: Clean up allocated memory to prevent leaks
3. **Security**: Only request necessary permissions
4. **Compatibility**: Test with different Moosync versions
5. **Documentation**: Provide clear README and code comments

### SDK and Tools:

- **Rust SDK**: `moosync-extension-sdk` crate (coming soon)
- **TypeScript definitions**: Available in the main repository
- **Development CLI**: Tools for building, testing, and packaging extensions
- **Extension store**: Official repository for sharing extensions

### Example Extension:

See the `examples/` directory in the main repository for complete extension examples in different languages.
