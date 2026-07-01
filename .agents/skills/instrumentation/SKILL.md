---
name: instrumentation
description: Hermetic tracing instrumentation rules and validation. Covers the requirement for #[tracing::instrument(level = "debug", skip_all)] on all function definitions, run verification via bazel run //tools:check_instrument, and automatic pre-commit hook integration.
---

# Instrumentation Skill

## Description

The **Instrumentation** skill enforces logging best practices in the Moosync repository. To preserve stack traces and keep execution flows observable, every function definition in Rust source files must be decorated with the `#[tracing::instrument]` attribute.

## Requirements

1. **Instrumentation on All Functions**: Every function definition (with a body) must have a `#[tracing::instrument]` attribute, unless it is:
   - A test function or inside a test module (e.g., `mod tests`).
   - A trait declaration signature (without a body).
   - A common built-in trait method: `main`, `default`, `from`, `into`, `try_from`, `fmt`, `clone`, `drop`.
2. **Skip All Parameters (`skip_all`)**: To prevent compilation errors caused by non-`Debug` types (such as JNIEnv, callbacks, closures, platform context objects, or `dyn` trait objects), always skip all function parameters using `skip_all`.

### Preferred Pattern:
```rust
#[tracing::instrument(level = "debug", skip_all)]
pub fn my_function(param1: NonDebugClosure, param2: JniContext) -> Result<(), MyError> {
    // ...
}
```

---

## Commands

| Command | Description | Example |
|---|---|---|
| **Verify Instrumentation** | Runs recursive checks on all `.rs` files to find missing instrumentation. | `bazel run //tools:check_instrument` |
| **Fix Skips/Instrumentation** | Programmatically updates all `tracing::instrument` attributes to standardized `skip_all`. | `python3 tools/fix_instrumentation_skips.py` |

---

## Git Pre-Commit Hook Integration

The instrumentation check is integrated into the Git pre-commit hook:
- When you stage any `.rs` or `.slint` files and run `git commit`, the pre-commit hook (located in `.git/hooks/pre-commit`) runs `bazel run //tools:check_instrument`.
- If any function is missing the instrumentation attribute, the pre-commit hook fails with exit code 1 and blocks the commit, reporting the exact files and line numbers needing attention.
