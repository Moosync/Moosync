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
| **instrumentation** | Tracing instrumentation rules and validation tools. | Any function changes or additions requiring observability. See `.agents/skills/instrumentation/SKILL.md`. |
| **tests**   | Unit and smoke testing rules, 3-section test layout, and hermetic execution. | Writing or refactoring tests. See `.agents/skills/tests/SKILL.md`. |

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

5. Use `bazel run //tools:format -- <files>` to format Rust (`.rs`), Slint (`.slint`), Bazel (`BUILD`, `MODULE.bazel`, `.bzl`), and Protobuf (`.proto`) files. This is wired as the git pre-commit hook via `.git/hooks/pre-commit`. Do not invoke rustfmt, slint-lsp, buildifier, or buf directly — they are resolved through Bazel runfiles.
6. No Clippy lints exist in this project. Use `bazel query` to discover available targets for a crate; test suites are defined under the root `BUILD` (e.g., `core_tests`).
7. Use `bazel run //tools:extract_translations` to extract translatable strings from Slint files into `ui/slint/locales/slint_app.pot`.
12. **Nesting and Control Flow**: Conditions should not be nested and an early return pattern should be preferred (e.g., using `let else` or flat early exits). Avoid using `else` blocks wherever possible, and avoid using `continue` inside loops to maintain clear and readable control flow.
13. **Single Responsibility & No Else Blocks**: Avoid `else` blocks entirely to reduce cognitive load and simplify control flow. Each function must perform only one task. Split distinct logic paths into separate, single-purpose helper functions and have the caller function orchestrate or delegate using flat early returns.
14. **Simple & Obvious Naming**: Keep function names simple and obvious. Do not repeat context from the struct or module name inside function names (e.g., in `PlaylistContentPageHandler`, use `fetch_local` or `fetch_local_songs` instead of `fetch_local_playlist_songs`).

### Tracing Instrumentation

7. Every Rust function definition must be decorated with `#[tracing::instrument(level = "...", skip_all)]` to ensure stack traces are preserved. All parameters must be skipped using `skip_all` to avoid compiler failures with non-`Debug` types.
8. Validate instrumentation by running `bazel run //tools:check_instrument`. This check runs automatically in the Git pre-commit hook.

### Plugin System

9. New plugins are registered by adding types to the `generate_plugin_system!()` macro call in `core/state_manager/src/lib.rs`. The macro generates all registry code — do not create plugin registration boilerplate manually.
10. **Context pattern** for platform-specific code: each major crate has a `context/` submodule (`preferences/context/keyring_context.rs`, `mpris/context/mod.rs`). New platform implementations should follow this pattern.

### Code Style & UI Modals

- **No Fully-Qualified Inline Paths**: Always import types and functions at the top of the file (`use ...;`). Never use inline fully-qualified paths (e.g., `songs_proto::...`, `crate::utils::...`) in code bodies.
- **No Abbreviations**: Do not abbreviate domain objects or variables (e.g., use `state_manager`, never `sm`).
- **Avoid Unnecessary Cloning**: Mutate collections in place rather than cloning expensive structures (like `Song`).
- **Control Flow Simplicity**: Prefer simple, direct `if let Some(...) = ...` over awkward `let Some(...) = ... else { return; }` inversions. Avoid `else` blocks and `continue` inside loops.
- **Single Responsibility & Thin UI Handlers**: Split distinct logic paths into separate helper functions; keep page handlers thin by moving multi-step coordination logic into `utils.rs`.
- **Slint Naming Conventions**: All Slint variables, properties, callbacks, and functions must strictly use `snake_case` (never `kebab-case`).
- **Slint Modal Lifecycle**: Reusable modals inherit from `Modal` and live in `ui/slint/src/common/`. The parent controls visibility conditionally (`if show_modal: MyModal { close => { show_modal = false; } }`). Rely on `Modal`'s built-in `callback close();` — backdrop clicks and dialog action buttons trigger `root.close()`. Do not create custom `is_open` properties on modals.

### Git & Version Control

11. Never use destructive git commands (`git reset`, `git checkout`). Only use `git diff` and `git status`.

---
