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
All unit tests must follow the standard 3-section structure (setup -> action -> assertions) separated by empty newlines:

```rust
#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_feature_specific_behavior() {
    let tmp = TempDir::new("test_context").unwrap();
    let handler = MyHandler::new(tmp.path());

    let result = handler.process_item("item_id");

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ExpectedState);
}
```

- Always place assertions at the very end of the test function.
- Avoid trailing cleanup statements after assertions — use RAII types like `tempdir::TempDir` so cleanup happens automatically on scope exit.
- **Never add comments to tests** unless you are performing some obscure action that is unrelated to the test or too complicated to understand in layman's terms. Let empty lines naturally demarcate test sections.

### 3. Granular, Single-Purpose Test Cases
- **Rule #1: Only test one thing at a time**: Each unit test must test a single action or condition. Do not combine multiple unrelated workflows into one test.
- **Rule #2: Dedicated test case for each code path**: Separate error paths, edge cases, and success paths into distinct, descriptive test functions (e.g. `test_login_success`, `test_login_invalid_password`).
- **Rule #3: Don't assert on intermediate setup steps**: Focus assertions on the output and final state of the function under test.
- **Rule #4: Never assert on no-ops**: Functions that are empty or no-ops (`{}`) must not have unit tests asserting nothing changed.

### 4. Parameterized Tests & Fixtures with `rstest`
- **Always use `rstest`**: Use `rstest` for all fixtures (`#[fixture]`) and parameterized test cases (`#[case(...)]`).
- **Never use `test-case`**: Do not import or add `@crates//:test-case`. `rstest` provides complete parameterized testing support natively.
- **Async Tests with Fixtures**: Annotate async tests with both `#[rstest]` and `#[tokio::test]`.
- **Multi-threaded Tokio Tests**: When background tasks or timers must run concurrently with a blocking UI loop, use proc macro directives `#[tokio::test(flavor = "multi_thread", worker_threads = 2)]` rather than manually constructing runtimes in code.

```rust
#[rstest]
#[case(Pages::AllSongs, AppPage::AllSongs)]
#[case(Pages::Albums, AppPage::Albums)]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_app_page_from_pages(#[case] page: Pages, #[case] expected: AppPage) {
    assert_eq!(AppPage::from(page), expected);
}
```

### 5. State-Agnostic and Hermetic Execution
- Tests must never depend on execution order or runner flags (e.g., do not rely on `RUST_TEST_THREADS=1` in Bazel).
- Use `tempdir::TempDir` for isolated temporary directories.
- **Slint Unit Tests**: Must use `i_slint_backend_testing::init_no_event_loop()`. This can be initialized in fixtures.
- **Slint Integration Tests**: Must use the event loop backend (`i_slint_backend_testing::init_integration_test_with_system_time()`), initialized once per process (never using `slint::run_event_loop()`).
- **Slint Integration Test Structure**: Write test logic as plain async functions taking `(main_window: &'static MainWindow, state_manager_fixture: TestSlintSmContext)`. Never write inline `spawn_local` or `run_event_loop` boilerplate inside test functions; use the `integration_test!` macro runner.

### 6. Slint Integration Testing & Three-Tier Verification
- **Rule #1: Single CUJ per Integration Test**: Every integration test must focus strictly on one Critical User Journey (CUJ) or user action (e.g. playing a track, pausing, skipping next, clearing the queue, toggling an extension provider, opening settings). Use parameterized tests (`rstest` `#[case(...)]`) when testing variations of the same user journey.
- **Rule #2: Three-Tier Verification Strategy**: Every integration test must verify across all applicable layers:
  1. **Tier 1 (Rendered UI Tree Assertions - MANDATORY)**: Must assert that elements actually exist, are visible, or have been completely unmounted in the rendered Slint UI tree using `i_slint_backend_testing::ElementHandle` (e.g., `ElementHandle::find_by_accessible_label(main_window, label)` returning valid elements with `len() == 1` when visible and `len() == 0` when unmounted/removed).
  2. **Tier 2 (Slint Global Props & Models)**: Must assert reactive properties and data vectors on global property structs (`main_window.global::<Props>().get_*()`).
  3. **Tier 3 (Core Backend Crate State)**: Must assert real backend state inside `core/player` (`PlayerHandler`), `core/database` (`Database`), `core/preferences` (`PreferenceConfig`), and `core/extensions` (`ExtensionHandler`).
- **Rule #3: Checking Props is NOT a UI Assertion**: Inspecting `ModelRc`, `VecModel`, or global property structs only proves that the Rust state handlers ran. A test only qualifies as asserting the UI when it verifies element handles (`ElementHandle`) in the rendered Slint UI tree.
- **Rule #4: Real Components Over Mocks**: Integration tests must always use real production components (real `StateManager`, real SQLite `Database`, real `ExtensionHandler` with WASM runtime, real `PreferenceConfig`, real `PlayerHandler`). Hardware-dependent drivers that cannot run in headless test environments (like rodio audio hardware) must use neutral test contexts (like `DummyAudioPlayerContext`).
- **Rule #5: UI Event Driving**: Simulate real user actions by clicking `ElementHandle`s (using `handle.single_click(slint::platform::PointerEventButton::Left).await`) rather than invoking raw Rust callbacks directly whenever feasible.
- **Rule #6: Accessible Labels on Interactive Controls**: All interactive buttons, icon buttons, and inputs in `.slint` files must have explicit `@tr("...")` accessible labels (`accessible-label: @tr("...");`) so they are discoverable and testable with `ElementHandle`.
- **Rule #7: No Visibility Changes for Tests (Use Test-Only Traits)**: Never change visibility (`pub`) on private struct fields, methods, or internals solely to support tests. If a component requires specialized state inspection or verification beyond its public API in tests, define a test-only trait (e.g. `#[cfg(test)] trait ComponentTestExt`) in test modules and implement it for the component.

### 7. Separation of Smoke Tests
- Pure construction or plugin initialization tests (`Plugin::init`, `new`) that verify initialization does not panic without operational assertions must be placed in separate files named `filename_test_smoke.rs` (e.g. `lib_test_smoke.rs`, `remote_test_smoke.rs`).

### 8. Build & Instrumentation Rules
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

### Slint UI Integration Test (Three-Tier Assertion)
```rust
#[tracing::instrument(level = "debug", skip_all)]
async fn do_playback_pause_song(
    main_window: &'static MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager: &'static StateManager = Box::leak(Box::new(sm));
    setup_test_context(state_manager).await;
    setup_ui(main_window, state_manager);
    let song = SongModel {
        id: "pause_1".into(),
        title: "Pause Song".into(),
        playback_url: "https://example.com/pause_1".into(),
        ..Default::default()
    };
    main_window.global::<AppCallbacks>().invoke_play_song(song);
    let _ = wait_until(|| main_window.global::<PlayerProps>().get_playing()).await;

    let pause_buttons: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Pause").collect();
    assert_eq!(pause_buttons.len(), 1);
    pause_buttons[0]
        .single_click(slint::platform::PointerEventButton::Left)
        .await;

    let paused = wait_until(|| !main_window.global::<PlayerProps>().get_playing()).await;
    assert!(paused);
    // Tier 1: Rendered UI Tree assertion
    let play_buttons: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Play").collect();
    assert_eq!(play_buttons.len(), 1);
    assert!(play_buttons[0].is_valid());
    // Tier 2: Slint Props assertion
    assert!(!main_window.global::<PlayerProps>().get_playing());
    // Tier 3: Core Crate State assertion
    let ph = state_manager.get_player_handler().await;
    assert_eq!(ph.get_player_state(), PlayerState::Paused);
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
