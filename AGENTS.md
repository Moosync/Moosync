# Moosync Project Agents and Skills

## Overview

Moosync is a cross-platform music player written in Rust, built entirely with **Bazel** using `rules_rust`. All build operations must go through Bazel — never use Cargo directly. The workspace uses bzlmod (`MODULE.bazel`) for dependency management.

Don't try to explore the entire project. Understand the context of the task and refer to the moosync skill to determine which files to read.

---

## Skill References

| Skill       | Purpose                                                                                           | When to Use                                                                                               |
| ----------- | ------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------- |
| **rust**    | Hermetic Rust crate building, testing, and formatting via Bazel targets.                          | Any Rust code changes — build, test, format. See `.agents/skills/rust/SKILL.md`.                          |
| **bazel**   | Hermetic build/test/package commands; dependency management with bzlmod. Patching external rules. | Cross-project builds, packaging (`@pkg//:all`), adding dependencies. See `.agents/skills/bazel/SKILL.md`. |
| **moosync** | Project structure — crate boundaries, UI layout, plugin system, platform-specific code patterns.  | Adding new features; understanding where to put new code. See `.agents/skills/moosync/SKILL.md`.          |

---

## Core Rules

### Mandatory rules

Donot

### Build System

1. **Always use Bazel** — never invoke Cargo directly. Use `bazel build //path/to/target` and `bazel test //path/to/target_test`.
2. **All builds must be hermetic** — no environment variables or system libraries unless explicitly declared in a `BUILD.bazel` file.
3. If a Bazel target fails, the issue is with your code or missing dependencies — not with Bazel itself.
4. **NEVER run `bazel clean`** unless explicitly instructed by user. It invalidates cached outputs and masks real build issues.

### Rust Operations

5. Use `bazel run //tools:format -- <files>` to format Rust (`.rs`) and Slint (`.slint`) files. This is wired as the git pre-commit hook via `.git/hooks/pre-commit`. Do not invoke rustfmt or slint-lsp directly — they are resolved through Bazel runfiles.
6. No Clippy lints exist in this project. Use `bazel query` to discover available targets for a crate; test suites are defined under the root `BUILD` (e.g., `core_tests`).

### Plugin System

7. New plugins are registered by adding types to the `generate_plugin_system!()` macro call in `core/state_manager/src/lib.rs`. The macro generates all registry code — do not create plugin registration boilerplate manually.
8. **Context pattern** for platform-specific code: each major crate has a `context/` submodule (`preferences/context/keyring_context.rs`, `mpris/context/mod.rs`). New platform implementations should follow this pattern.

### Git & Version Control

9. Never use destructive git commands (`git reset`, `git checkout`). Only use `git diff` and `git status`.

---
