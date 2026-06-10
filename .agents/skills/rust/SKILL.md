---
name: rust
description: Hermetic Rust crate building, testing, and formatting via Bazel targets. Covers build/test commands through rules_rust, formatting with bazel run //tools:format, test suites defined in root BUILD, and dependency management via MODULE.bazel crates.io integration.
---

# Rust Skill

## Description

The **Rust** skill provides a set of reusable commands for building, testing, and formatting Rust crates in the Moosync repository. All operations use Bazel's hermetic `rules_rust` toolchain — Cargo is never invoked directly.

## Commands

| Command          | Description                                                                          | Example                                                          |
| ---------------- | ------------------------------------------------------------------------------------ | ---------------------------------------------------------------- |
| **Build crate**  | Build a specific Rust library or binary target via Bazel.                            | `bazel build //core/player:player`                               |
| **Run tests**    | Execute unit tests for a crate through Bazel.                                        | `bazel test //core/database:database_test`                       |
| **Format files** | Run hermetic formatter on staged `.rs` and `.slint` files via the pre-commit target. | `bazel run //tools:format -- path/to/file.rs path/to/file.slint` |

## Usage Patterns

1. **Quick build of a feature module** — identify the target label in the crate's `BUILD.bazel` file and run the _Build crate_ command.
2. **Test after change** — run _Run tests_ to catch regressions before committing. Test suites are aggregated under `core_tests` at the root level.
3. **Formatting** — invoke _Format files_ as part of a Git pre-commit hook (see `.git/hooks/pre-commit`). The formatter resolves hermetic rustfmt and slint-lsp through Bazel runfiles.

## Example Workflow

```bash
# Build the player module
bazel build //core/player:player

# Run tests for the database crate
bazel test //core/database:database_test

# Format specific files (pre-commit hook pattern)
bazel run //tools:format -- core/player/src/lib.rs ui/slint/app.slint
```

## Rules_rust Integration Notes

- All Rust dependencies are declared in `modules/crates.MODULE.bazel` via the `crates.io` integration. Do not manually edit lockfiles.
- New crates should use `rust_library`, `rust_binary`, or `rust_shared_library` from `@rules_rust//rust:defs.bzl`.
- Build scripts (build.rs) are declared with `cargo_build_script()` from `@rules_rust//cargo:cargo_build_script.bzl`.
