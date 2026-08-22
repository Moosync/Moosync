---
name: moosync
description: Project-wide helpers for adding new features to Moosync. Covers file structure, crate-by-crate breakdown with purposes and code locations, UI layout under ui/slint/, plugin registration via generate_plugin_system!, and the context pattern for platform-specific implementations.
---

# Moosync Skill

## File Structure Overview

```
Moosync/                          # Root workspace (Bazel MODULE.bazel)
├── core/                         # All core logic – Rust crates
│   ├── database/                 # SQLite DB layer + migrations + cache
│   ├── extensions/               # WASM plugin system (Extism runtime)
│   ├── file_scanner/             # Music library scanner
│   ├── lyrics/                   # Lyrics fetching service
│   ├── mpris/                    # MPRIS2 media controller
│   ├── player/                   # Audio playback engine (rodio)
│   ├── plugin_macro/             # Plugin system macro generator
│   ├── preferences/              # User preference storage
│   ├── spotify_player/           # Spotify Connect integration
│   ├── state_manager/            # Central orchestrator + plugin registry
│   ├── theme_macro/              # Theming macros
│   ├── themes/                   # Theme rendering
│   └── types/                    # Shared types, errors, platform bindings
├── ui/slint/                     # UI – Slint .slint files compiled to Rust
│   ├── app.slint                 # Main window shell
│   ├── callbacks.slint           # Global callback handlers
│   ├── constants.slint           # Constants and global state
│   ├── main_content/             # 17 page handler .slint files (albums, artists, search, etc.)
│   ├── bottombar/                # Bottom bar components
│   ├── common/                   # Reusable widgets (buttons, cards, lists)
│   ├── icons/                    # SVG icon assets
│   └── settings/                 # Settings pages
├── android/                      # Android app Gradle build wrapper
├── package/                      # Distribution packages (AUR, DEB, Fedora, etc.)
├── tools/                        # Bazel helpers and Rust toolchains
├── patches/                      # Patch files for external rules
└── modules/                      # bzlmod sub-files (crates, toolchains, deps)
```

---

## Crate-by-Crate Breakdown

### `core/database` – Database Layer

- **Purpose:** SQLite database with SQLx-style migrations. Handles song storage, playlist management, caching, and library item tracking.
- **Key files:** `database.rs`, `migrations/`, `cache.rs`, `utils.rs`
- **Where to add code:** New models → `database.rs`; new migrations → create timestamped folder in `migrations/`.

### `core/player` – Audio Player Engine

- **Purpose:** Audio playback using rodio. Manages queue, shuffle, repeat modes (None/Shuffle/RepeatOne), volume control, song resolution, and playback state callbacks.
- **Key files:** `lib.rs` (PlayerHandler with 20+ methods), `audio_source.rs`, `mux_player.rs`, `rodio/`
- **Where to add code:** New audio source types → extend `audio_source.rs`; player logic → modify `lib.rs` or `mux_player.rs`.

### `core/extensions` – Plugin System

- **Purpose:** Extension plugin manager using Extism WASM runtime. Handles installation, activation, command execution, and UI preference registration for third-party plugins.
- **Key files:** `lib.rs`, `ext_runner.rs`, `models.rs`, `context/extism_context.rs`
- **Where to add code:** New extension types → `models.rs`; command handling → `ext_runner.rs`.

### `core/file_scanner` – Library Scanner

- **Purpose:** Scans local music files, creates playlists from scan results, extracts thumbnails, manages artist split characters. Platform-specific via context pattern.
- **Key files:** `lib.rs`, `context/android.rs`, `context/desktop.rs`
- **Where to add code:** Scanner logic → `lib.rs`; platform specifics → `context/`.

### `core/lyrics` – Lyrics Fetcher

- **Purpose:** Fetches lyrics for currently playing songs.
- **Key files:** `lib.rs`
- **Where to add code:** New providers → extend the existing fetcher interface in `lib.rs`.

### `core/mpris` – MPRIS2 Controller

- **Purpose:** Desktop/media player integration via MPRIS2 protocol. Platform-specific: Android (`mpris_android.rs`), Windows (`win32.rs`), Linux/Souvlaki (`context/mod.rs`).
- **Key files:** `lib.rs`, `context/mod.rs`, platform-specific files per OS.
- **Where to add code:** Context implementations → `context/`; platform bindings → separate file per target OS.

### `core/preferences` – Preferences Storage

- **Purpose:** User preference storage with keyring-backed persistence on desktop, file-based fallback elsewhere. Platform-specific via context pattern.
- **Key files:** `lib.rs`, `preferences.rs`, `context/keyring_context.rs`
- **Where to add code:** New preference keys → `preferences.rs`; context providers → `context/`.

### `core/state_manager` – Central State Manager (DO NOT MODIFY DIRECTLY FOR PLUGINS)

- **Purpose:** Orchestrates all plugins via auto-generated plugin registry. Manages interceptors, reply handlers, and runtime setup for extensions, themes, player, and scanner.
- **Key files:** `lib.rs`, `reply_handler.rs`, `interceptors/mod.rs`
- **Where to add code:** New plugins are registered automatically by adding types to `generate_plugin_system!()` macro in `lib.rs`. Do not manually create plugin registration boilerplate.

### `core/themes` & `core/theme_macro` – Theming System

- **Purpose:** Theme rendering and theming macros for consistent UI styling.
- **Key files:** `themes.rs`, test files.
- **Where to add code:** New themes → `themes.rs`; theme macros → extend `theme_macro/src/lib.rs`.

### `core/plugin_macro` – Plugin Macro Generator

- **Purpose:** Generates the plugin registry boilerplate from a list of types via `generate_plugin_system!()` macro.
- **Key files:** `lib.rs`
- **Where to add code:** Only modify if changing core plugin architecture.

### `core/types` – Shared Type Definitions

- **Purpose:** All shared types, errors, platform bindings (Android), MPRIS types, provider interfaces, UI state models, song extensions, scan progress utilities.
- **Key files:** Submodules: `src/errors/`, `src/android/`, `src/mpris/`, `src/providers/`, `src/ui/`
- **Where to add code:** New shared types → appropriate submodules in `src/`.

### `core/spotify_player` – Spotify Integration

- **Purpose:** Spotify Connect via librespot. Handles player state sync with main audio engine, protocol handling, and provider abstraction.
- **Key files:** `lib.rs`, `player.rs`, `canvaz.rs`, `spirc.rs`, `utils.rs`
- **Where to add code:** New protocol handlers → extend `player.rs`; provider logic → `canvaz.rs`, `spirc.rs`.

---

## UI Structure (`ui/slint/`)

The UI uses Slint declarative markup (`.slint`) compiled into Rust. Main entry point is `lib.rs` which calls `run()`.

| Path                                 | Purpose                                                                                        | Where to Add New Code                                          |
| ------------------------------------ | ---------------------------------------------------------------------------------------------- | -------------------------------------------------------------- |
| `app.slint`                          | Main window shell and layout                                                                   | Modify for global UI changes                                   |
| `callbacks.slint`, `constants.slint` | Shared handlers, constants, global bindings                                                    | Extend for new shared state                                    |
| `main_content/` (17 files)           | Primary pages – albums, artists, all songs, explore, genres, library, playlists, queue, search | New page → add `.slint` file + implement `PageHandler` in Rust |
| `bottombar/`                         | Playback controls, progress bar, track info, volume                                            | Modify for playback UI changes                                 |
| `common/` (8 widgets)                | Reusable components – buttons, cards, lists, modals, wrappers                                  | New widget → add `.slint` file + export from Rust              |
| `icons/`                             | SVG icon assets                                                                                | Add new `.svg` files                                           |
| `settings/`                          | Settings pages – extensions, paths, system, themes                                             | New setting page → add to `settings/`                          |

---

## Plugin Registration Pattern

New plugins are registered by adding types to the `generate_plugin_system!()` macro in `core/state_manager/src/lib.rs`:

```rust
plugin_macro::generate_plugin_system!(
    preferences::preferences::PreferenceConfig,
    database::Database,
    file_scanner::ScannerHolder,
    lyrics::LyricsFetcher,
    extensions::ExtensionHandler,
    player::PlayerHandler,
    themes::themes::ThemeHolder,
    mpris::MprisHolder
);
```

The macro generates all registry code automatically. Do not manually create plugin registration boilerplate.

---

## Platform-Specific Code Pattern

Each major crate uses a `context/` submodule for target OS abstractions:

- `preferences/context/keyring_context.rs` – keyring-backed preferences on desktop
- `mpris/context/mod.rs` – Souvlaki (Linux), Android bindings, Windows dummy
- `file_scanner/context/android.rs`, `desktop.rs` – platform scanner implementations

New platform-specific implementations should follow this pattern: create a context trait in the crate's `lib.rs`, then implement it per-target OS in `context/`.

---

_This file is automatically generated from the current repository state._
