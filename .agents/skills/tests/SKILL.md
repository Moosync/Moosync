---
name: tests
description: Guidelines and rules for writing unit, integration, and smoke tests across Moosync. Covers test file placement, strict 3-section layout (setup -> function -> assertions), state isolation, Slint UI testing, and smoke test separation.
---

# Testing Skill

## Overview

Moosync enforces strict conventions for unit and integration testing. Every Rust file in the project must have its own dedicated test file, each function should ideally have dedicated test coverage for all code paths, and all tests must be completely state-agnostic and hermetic.

---

## Core Testing Rules

### 1. File & Function 1:1 Coverage
- Every source file `path/to/foo.rs` must have a corresponding test file `path/to/foo_test.rs`.
- Each function should ideally be covered with dedicated unit tests for every logical code path (success, error, edge cases).
- All logical code (protobuf conversion, state manager reply handlers, DB queries, UI page handlers, etc.) must be covered with unit tests. No exceptions.
- **Do NOT make private functions public for tests**: Test private functions through the exposed public API and assert the end results.

### 2. Strict 3-Section Test Layout
All unit tests must follow the standard 3-section structure separated by empty newlines:

```rust
#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_feature_specific_behavior() {
    // 1. [test environment setup]
    let tmp = TempDir::new("test_context").unwrap();
    let handler = MyHandler::new(tmp.path());

    // 2. [function that is being tested]
    let result = handler.process_item("item_id");

    // 3. [assertions]
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ExpectedState);
}
```

- Always place assertions at the very end of the test function.
- Avoid trailing cleanup statements after assertions — use RAII types like `tempdir::TempDir` so cleanup happens automatically on scope exit.

### 3. Granular, Single-Purpose Test Cases
- **Rule #1: Only test one thing at a time**: Each unit test must test a single action or condition. Do not combine multiple unrelated workflows into one test.
- **Rule #2: Dedicated test case for each code path**: Separate error paths, edge cases, and success paths into distinct, descriptive test functions (e.g. `test_login_success`, `test_login_invalid_password`).
- **Rule #3: Don't assert on intermediate setup steps**: Focus assertions on the output and final state of the function under test.
- **Rule #4: Never assert on no-ops**: Functions that are empty or no-ops (`{}`) must not have unit tests asserting nothing changed.

### 4. State-Agnostic and Hermetic Execution
- Tests must never depend on execution order or runner flags (e.g., do not rely on `RUST_TEST_THREADS=1` in Bazel).
- Use `tempdir::TempDir` for isolated temporary directories.
- In Slint UI tests, execute inside `crate::test_utils::run_test` or `crate::test_utils::run_async_test` and reset UI model state before each test.

### 5. Separation of Smoke Tests
- Pure construction or plugin initialization tests (`Plugin::init`, `new`) that verify initialization does not panic without operational assertions must be placed in separate files named `filename_test_smoke.rs` (e.g. `lib_test_smoke.rs`, `remote_test_smoke.rs`).

### 6. Build & Instrumentation Rules
- **Explicit files in BUILD**: Never use `glob()` in `BUILD` files; list every `.rs`, `*_test.rs`, and `*_test_smoke.rs` file explicitly in `srcs`.
- **Tracing Instrumentation**: Every test function definition must be decorated with `#[tracing::instrument(level = "debug", skip_all)]`. Validate with `bazel run //tools:check_instrument`.
- **Formatting**: Run `bazel run //tools:format` on modified files before committing.

---

## Example Test Patterns

### Standard Async Unit Test
```rust
use tempdir::TempDir;
use types::plugin::PluginContext;

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_database_insert_song_success() {
    let tmp = TempDir::new("db_test").unwrap();
    let db = Database::new_with_path(tmp.path()).unwrap();
    let song = create_test_song("song_1");

    let result = db.insert_song(song);

    assert!(result.is_ok());
    assert_eq!(db.get_song("song_1").unwrap().title, "Test Title");
}
```

### Slint UI Unit Test
```rust
use slint::{ComponentHandle, Model, ModelRc};
use crate::{
    AlbumsPageProps, MainWindow, main_content::albums::AlbumsPageHandler,
    test_utils::run_async_test,
};

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_albums_page_handler_on_show() {
    run_async_test(|| async move {
        let main_window = Box::leak(Box::new(MainWindow::new().unwrap()));
        main_window.global::<AlbumsPageProps>().set_albums(ModelRc::default());
        let handler = AlbumsPageHandler::new(main_window, mock_state_manager());

        handler.on_show();

        assert_eq!(
            main_window.global::<AlbumsPageProps>().get_albums().row_count(),
            0
        );
    });
}
```

### Smoke Test (`filename_test_smoke.rs`)
```rust
use tempdir::TempDir;
use types::plugin::{Plugin, PluginContext};
use crate::Database;

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_database_plugin_init() {
    let tmp = TempDir::new("db_smoke").unwrap();
    let context = PluginContext {
        data_dir: tmp.path().to_path_buf(),
        cache_dir: tmp.path().to_path_buf(),
        tmp_dir: tmp.path().to_path_buf(),
        #[cfg(target_os = "android")]
        android_context: types::android::AndroidJNIContext::default(),
    };

    let plugin = Database::init(&context);
    let _guard = plugin.blocking_read();
}
```
