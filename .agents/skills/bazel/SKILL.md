---
name: bazel
description: Hermetic Bazel build, test, and packaging commands for the Moosync project. Covers building all targets through rules_rust, running tests, creating distribution packages via @pkg//:all, dependency management with bzlmod (MODULE.bazel), sandbox-safe builds, and patching external rules.
---

# Bazel Skill

## Description

The **Bazel** skill wraps common Bazel commands used throughout the Moosync repository. It enforces hermetic builds and provides quick shortcuts for building targets, running tests, and packaging the entire project.

> _Hermeticity_ is a core requirement: all dependencies must be declared in `BUILD.bazel` files or fetched via `rules_rust`. This skill avoids accidental use of `bazel clean` or non-hermetic commands.

## Commands

| Command          | Description                                                                                   | Example                                    |
| ---------------- | --------------------------------------------------------------------------------------------- | ------------------------------------------ |
| **Build target** | Build any Bazel label.                                                                        | `bazel build //core/player:player`         |
| **Test target**  | Run tests for a given target.                                                                 | `bazel test //core/database:database_test` |
| **Package all**  | Build distribution packages defined under the `package/` directory (AUR, DEB, Flatpak, etc.). | `bazel build @pkg//:all`                   |

## Usage Guidelines

1. **Never run `bazel clean`** unless explicitly requested by a user; it can invalidate cached outputs and mask real build issues.
2. **Use the `@pkg` repository** to trigger packaging for all supported platforms.
3. For incremental builds, rely on Bazel's caching rather than manual cleaning.
4. Never add extra flags or change the platform while building bazel rules unless explicitly specified by the user.

## Example Workflow

```bash
# Build a core crate
bazel build //core/player:player

# Run tests (aggregated under core_tests at root level)
bazel test //core/database:database_test

# Package all distribution artifacts
bazel build @pkg//:all
```

## Bzlmod Dependency Management

- All dependencies are managed through `MODULE.bazel` and included sub-files under `modules/`.
- Rust crates: declared in `modules/crates.MODULE.bazel` via the `crates.io` integration.
- Toolchains: registered in `modules/rust_toolchains.MODULE.bazel`.

## Patching External Rules

When patching external rules, always:

1. Generate patches via `git diff` against the clean remote repository commit.
2. Place patches under `patches/` and expose them via `exports_files()` in a `BUILD` file.
3. Configure `git_override` with `patches` and `patch_strip` attributes in `MODULE.bazel`.
