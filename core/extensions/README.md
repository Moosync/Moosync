# `extensions`

Manages Moosync WASM extensions: discovery, package staging, OpenPGP + SHA-256 verification, sandboxing, and IPC dispatch.

## UI Contract

`ExtensionsPageHandler` (`ui/slint/src/settings/extensions.rs`) interacts with `ExtensionHandler` through events and commands. Queue modal prompts in the UI when `ExtensionDetailsState.show_modal` is already `true` so concurrent prompts do not overwrite each other.

### Event Listeners (register during UI initialization)

| Event | Payload | UI Action |
| --- | --- | --- |
| `on_unsigned_extension` | `UnsignedExtension { manifest, manifest_path }` | Enqueue `ExtensionDetailsModal` with `is_unsigned = true` and `is_unsigned_start_prompt = true`. |
| `on_extensions_updated` | `()` | Re-query `get_all_extensions()` and `has_updates()`, then re-render the extensions list. |
| `on_accounts_updated` | `Option<String>` | Refresh extension accounts UI. |
| `on_preferences_updated` | `String` | Refresh extension preferences UI. |

### User Actions

| Flow | When | Calls |
| --- | --- | --- |
| Refresh registries | Startup (`ExtensionRegistriesHook`) or user refresh | `fetch_remote_manifests(registries)` |
| Inspect local `.msxt` | User picks a local package file | `inspect_extension_package(&path)` -> open `ExtensionDetailsModal` (`is_unsigned = (signature_status == Unsigned)`) |
| Install local / remote | User clicks Approve on install prompt | `install_extension(ExtensionInfo::LocalPath(path))` or `install_extension(ExtensionInfo::Remote(manifest))` |
| Allow unsigned on start | User clicks Approve on `is_unsigned_start_prompt` | `load_extension(&manifest_path, true)` |
| Reject unsigned on start | User clicks Reject on `is_unsigned_start_prompt` | Close modal; do not call `load_extension` |
| Update all | User clicks Update All | `update_all_extensions()` |
| Remove extension | User uninstalls a local extension | `remove_extension(package_name)` |

## Signing and Verification

### Package Layout

An unpacked extension directory or `.msxt` zip archive contains:

- `manifest.pb`: serialized `ExtensionManifest` with `moosync_extension = true`, reverse-domain `package_name` (3+ dot-separated lowercase segments, e.g. `app.moosync.spotify`), and `file_hashes` (`map<string, string>` of filename to SHA-256 hex).
- `manifest.pb.asc` (optional for local, required for remote): ASCII-armored detached OpenPGP signature over `manifest.pb`.
- `extension.lock`: serialized `ExtensionLockData { registry }` written at install time.

### Trusted Key Storage

- `core/extensions` defines `RegistryKeyStorage` (`get_registry_key`, `set_registry_key`, `remove_registry_key`, `list_registry_keys`) so it does not depend on `core/preferences`.
- `ExtensionRegistriesHook::on_startup` (`core/state_manager/src/hooks/registries.rs`) injects `PrefsRegistryKeyStorage`, storing keys in encrypted `PreferenceConfig` secure storage (`set_secure` / `get_secure`).
- During `fetch_remote_manifests`, `RemoteExtensions::fetch_registry` calls `store_registry_key_checked`:
  - First seen key for `registry_url` -> stored immediately.
  - Same PGP fingerprint -> no-op.
  - Different PGP fingerprint -> leaves old key intact and marks `registry_url` as key-changed; `get_registry_key` refuses installs from that registry until the user removes and re-adds it.
- Removing a registry from `EXTENSION_REGISTRIES` triggers `ExtensionHandler::remove_registry_key`, clearing its stored key via `set_secure(..., None)`.

### Verification Rules

`signing::verify_package_contents(extracted_dir, manifest, keys)` runs before staging and before spawning an `Extension`:

1. Computes SHA-256 for every entry in `manifest.file_hashes`; missing files fail with `InvalidSignature` and mismatches fail with `HashMismatch`.
2. If `manifest.file_hashes` is non-empty, any extra file in `extracted_dir` (other than `manifest.pb`, `manifest.pb.asc`, `extension.lock`) fails with `InvalidSignature`.
3. If `manifest.pb.asc` is absent, returns `SignatureStatus::Unsigned`.
4. If `manifest.pb.asc` is present, parses it (`InvalidSignature` if malformed) and checks it against `keys` (primary key and subkeys). A match returns `SignatureStatus::Verified` with `key_fingerprint`; no matching trusted key returns `SignatureStatus::Unsigned`.

### Runtime Verification

- `ExtensionHandlerInner::spawn_extensions` calls `spawn_single_extension(..., allow_unsigned = false)`. `Extension::new_verified` runs `validate_package_name` + `verify_package_contents`; unsigned extensions return `Err(ExtensionError::UnsignedExtension { manifest, manifest_path })`, which `ExtensionHandler::find_new_extensions` emits on `on_unsigned_extension`.

## Staging and Installation

1. **Acquire archive** (`ExtensionHandler::install_extension`):
   - `ExtensionInfo::Remote`: looks up the registry's cached key via `RemoteExtensions::get_registry_key`, downloads the `.msxt` via `RemoteExtensions::download_extension`, and verifies the archive's `sha256` if present.
   - `ExtensionInfo::LocalPath`: uses the local path directly (`registry = "local"`).
2. **Stage** (`Extension::stage_package`):
   - Extracts the archive into a unique `tmp_dir` subdirectory.
   - Decodes `manifest.pb`, verifies `moosync_extension` and `validate_package_name`, and runs `verify_package_contents`. Remote installs fail with `MissingSignature` unless `SignatureStatus::Verified`.
   - If `extensions_dir/<package_name>/manifest.pb` already exists, requires `new_version > old_version` (`DuplicateExtension` otherwise). If the installed extension has `manifest.pb.asc`, its signing key fingerprint must match the staged package's fingerprint (`UpdateSigningKeyMismatch` otherwise).
3. **Validate remote identity and stop old instance** (`ExtensionHandler::install_extension`):
   - For remote installs, asserts `staged.package_manifest.package_name` and `version` match the requested remote manifest (`InvalidSignature` on mismatch).
   - If `staged.should_remove_old`, removes the running instance from `ExtensionHandlerInner` via `send_remove_extension`.
4. **Finalize and load** (`Extension::finalize_install`):
   - Removes any old directory, moves `tmp_dir` into `extensions_dir/<package_name>`, and writes `extension.lock`.
   - Remote installs call `find_new_extensions()` (requiring signature verification). Local installs call `load_extension(&installed_manifest_path, true)` since the user already approved the local package modal.
