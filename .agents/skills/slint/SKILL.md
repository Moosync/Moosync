---
name: slint
description: Expert guidance for building, debugging, and working with Slint GUI applications. Covers the .slint markup language, project setup, debugging with the embedded MCP server, and language API bindings for Rust, C++, JavaScript, and Python.
---

# Slint Development Skill

Use this skill when building, debugging, or reviewing applications that use [Slint](https://slint.dev), a declarative GUI toolkit for native user interfaces across desktop, embedded, mobile, and web platforms.

## When to Use This Skill

Use this skill when the task involves:

- Writing or debugging `.slint` files
- Integrating Slint with Rust, C++, JavaScript, or Python
- Investigating layout, binding, rendering, or event-handling issues
- Enabling the Slint MCP server for runtime inspection and UI debugging
- Explaining or reviewing Slint-specific code patterns

## How to Help

When using this skill:

- Prefer idiomatic Slint patterns over manual UI workarounds
- Match guidance to the user's language binding and Slint version
- Watch for common pitfalls such as binding loops, missing layout constraints, and type mismatches
- Suggest the MCP server when runtime inspection or interaction would make debugging easier
- Prefer solutions that preserve Slint's declarative and reactive model

## The .slint Language

Slint UIs are written in `.slint` markup files. The language is declarative and reactive.

## Naming Conventions

Follow these project naming conventions when writing `.slint` and related Rust files:

1. **Components and Structs**: Use `PascalCase` for component names and struct types (e.g., `QueuePage`, `SongModel`, `TextInputGroup`, `ModalSize`).
2. **Properties, Variables, Callbacks, and Functions**: All Slint variables, properties, callbacks, and functions MUST be `snake_case` and NEVER `kebab-case` (e.g., `in property <bool> is_active;`, `in-out property <string> new_playlist_name;`, `callback remove_clicked();`, `save_queue_as_playlist(string)`).
3. **Rust Function Names**: Follow standard Rust `snake_case`, keep names simple and obvious, and avoid repeating struct/module context in function names (e.g., `save_queue` rather than `save_queue_page_queue_songs`).


## Project Setup

### Rust

```python
# BUILD.bazel
load("@rules_rust//rust:defs.bzl", "rust_binary")
load("@rules_rust//cargo:cargo_build_script.bzl", "cargo_build_script")

cargo_build_script(
    name = "build_script",
    srcs = ["build.rs"],
    # Ensure the build script watches the .slint files for rebuilds
    data = glob(["ui/**/*.slint"]),
    deps = [
        "@crates//:slint-build",
    ],
)

rust_binary(
    name = "my_app",
    srcs = ["main.rs"],
    deps = [
        ":build_script",
        "@crates//:slint",
    ],
)

```

```rust
// build.rs
fn main() {
    slint_build::compile("ui/main.slint").unwrap();
}

```

```rust
// main.rs
slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let app = MainWindow::new()?;
    // Set up callbacks, models, etc.
    app.run()
}

```

## Debugging Slint Applications

### Common Issues

1. **Binding loops**: A property depends on itself through a chain of bindings. The compiler warns about these. Break the cycle by introducing an intermediate property or restructuring.
2. **Elements not visible**: Check `width`, `height` (may be 0 if not in a layout), `visible`, `opacity`, and parent clipping.
3. **Layout sizing**: Elements outside layouts need explicit `width`/`height`. Inside layouts, they get sized automatically. Use `preferred-width`, `min-width`, `max-width` to constrain.
4. **Type mismatches**: `length` and `int`/`float` are different types. Use `1px * my_int` to convert, or `my_length / 1px` to get a number.
5. **Performance**: Use `ListView` (not `for` in `ScrollView`) for long lists because it virtualizes. Use `image-rendering: pixelated` only when needed. Avoid deeply nested opacity or clip layers.

### Debug Helpers

- `debug("message", expression)` prints to stderr at runtime
- `SLINT_DEBUG_PERFORMANCE=refresh_lazy,console` enables performance diagnostics
- Run with `SLINT_BACKEND=winit-skia` or other backend variants for testing

## MCP Server for AI-Assisted Debugging

Slint includes an embedded MCP (Model Context Protocol) server that lets you inspect and interact with a running Slint application in real time. The server provides tools for exploring the UI tree, taking screenshots, clicking elements, dragging, typing, and more.

Once enabled, an AI coding assistant can connect to the MCP endpoint to inspect and interact with the running UI.

### Enabling the MCP Server

**Step 1**: Build with `SLINT_EMIT_DEBUG_INFO=1` so that element IDs and source locations are preserved in the compiled output. Without this, elements will lack the debug metadata needed for meaningful introspection. Set `SLINT_MCP_PORT` to an available port when running via Bazel:

```sh
SLINT_EMIT_DEBUG_INFO=1 SLINT_MCP_PORT=9315 bazel run //:my_app

```

In a Bazel setup, ensure the `mcp` feature is enabled for the `slint` crate within your workspace dependencies (e.g., via `crate.annotation` in `bzlmod` or `annotations` in `crates_repository`). Since Bazel does not have a direct equivalent to Cargo's ad-hoc `--features` command-line flag, you must configure it ahead of time in your build graph.

**Step 2**: Connect to the running application's MCP server at `http://localhost:9315/mcp` using Streamable HTTP transport and use the available tools to inspect and interact with the UI.

When scripting or verifying the server from the command line, use `curl` — it is the most reliable approach for raw JSON-RPC. Prefer `curl` over built-in HTTP fetch tools, which agents sometimes reach for but which are less predictable for this use case:

```sh
# Initialize (confirms the server is up and prints available tools)
curl -s -X POST http://127.0.0.1:9315/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'

# List windows
curl -s -X POST http://127.0.0.1:9315/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"list_windows","arguments":{}}}'

# Take a screenshot (response contains a base64-encoded PNG in the "data" field)
curl -s -X POST http://127.0.0.1:9315/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"take_screenshot","arguments":{"windowHandle":{"index":"1","generation":"1"}}}}'

```

### Version Requirements

| Slint Version | MCP Support                                                                      |
| ------------- | -------------------------------------------------------------------------------- |
| < 1.17.0      | Not available                                                                    |
| >= 1.17.0     | Enable via the `mcp` crate feature in your Bazel Rust dependencies configuration |

### When to Suggest MCP

Suggest enabling the MCP server when the user is:

- Debugging layout or visual issues
- Trying to understand the runtime element hierarchy
- Testing interactions programmatically
- Verifying accessibility properties
- Diagnosing event handling problems

## Translations & Internationalization (i18n)

### Annotating Strings in `.slint` Files

Use `@tr(...)` to mark user-facing string literals for translation:

```slint
// Simple text
Text { text: @tr("Songs"); }

// String formatting
Text { text: @tr("Adjust the color for {}", item.name); }

// Plural forms
Text { text: @tr("I have {n} item" | "I have {n} items" % count); }
```

### Extracting Translatable Strings

A Bazel rule is provided to extract all `@tr(...)` strings into `ui/slint/locales/slint_app.pot`:

```bash
# Extract strings across all Slint files
bazel run //tools:extract_translations

# Or extract from specific Slint files
bazel run //tools:extract_translations -- ui/slint/src/app.slint
```

> **Note**: This rule uses `slint-tr-extractor` (installable via `cargo install slint-tr-extractor`).

### Bundled Translations Directory Structure

Translation catalogs follow the standard Gettext hierarchy under `ui/slint/locales/`:

```
ui/slint/locales/
├── slint_app.pot
├── de_DE/LC_MESSAGES/slint_app.po
├── es_ES/LC_MESSAGES/slint_app.po
├── fr_FR/LC_MESSAGES/slint_app.po
└── ...
```

### Build & Bundling Setup

1. **`build.rs`**:
   ```rust
   let config = slint_build::CompilerConfiguration::new()
       .with_bundled_translations("locales")
       .with_default_translation_context(slint_build::DefaultTranslationContext::None);
   slint_build::compile_with_config("src/app.slint", config).unwrap();
   ```

2. **`ui/slint/BUILD`**:
   - `build_script_env` must specify `"CARGO_PKG_NAME": "slint_app"`.
   - `data` in `cargo_build_script` and `compile_data` in `rust_library` must include `"locales/**/*.po"`.

## Documentation Reference

Full documentation for the latest version is at [https://slint.dev/docs](https://slint.dev/docs). Key sections:

- Language guide: concepts, syntax, and coding patterns
- Reference: elements, properties, types, and standard widgets
- Translations: [https://docs.slint.dev/latest/docs/slint/guide/development/translations/](https://docs.slint.dev/latest/docs/slint/guide/development/translations/)
- Language integrations: Rust, C++, Node.js, and Python API docs
- Tutorials: step-by-step guides for each language

The documentation can be found at `https://snapshots.slint.dev/master/docs/slint/`.

