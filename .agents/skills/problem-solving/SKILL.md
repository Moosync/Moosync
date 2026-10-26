---
name: problem-solving
description: Rules and workflow for diagnosing, reproducing, and fixing bugs. Covers reproducing issues with minimal unit or integration tests, strictly preserving existing test behavior, TDD bug fixing without altering the failing test, producing minimal diffs, and pruning unused code.
---

# Problem Solving Skill

## Overview

This skill defines the mandatory protocol for investigating, diagnosing, and resolving bugs, regressions, or unexpected behaviors in the Moosync repository. Every fix must follow a disciplined, test-driven reproduction lifecycle while ensuring minimal impact on the codebase.

---

## Core Rules & Protocol

### 1. Always Replicate the Issue with Tests
- Before modifying any implementation or business logic, write an automated test that reproduces the bug.
- **Choose the simpler test type**:
  - Use a **unit test** when the issue is isolated to a specific module, function, or data transformation.
  - Use an **integration test** when the issue involves component interactions, state manager communication, database queries, or UI event flows.
- Pick whichever test approach is simpler and fastest to reliably reproduce the failure mode.

### 2. Never Change the Behaviour of Existing Tests
- Existing tests represent system invariants and established regression barriers.
- **Never weaken, modify, or remove assertions** in existing tests to accommodate a change.
- Never alter the logic, test setup, or expected outcomes of pre-existing tests.
- If existing tests fail after your modifications, your change broke existing contracts or expectations. Fix your code to keep existing tests green.

### 3. Write a Broken Test, Then Fix the Code
- **Red phase**: First write the test that demonstrates the bug. Run the test with Bazel to confirm that it fails as expected.
- **Green phase**: Modify the source code to resolve the failure until the new test passes.
- **Never change the broken test's behaviour**: Once written, do not adjust the assertions or expectations of the reproduction test to match buggy or incomplete behavior. The test must remain unchanged while the implementation code is fixed around it.

### 4. Minimal Diffs & Smallest Possible Changes
- Every bug fix must be as minimal and targeted as possible.
- **DO NOT make large, unnecessary changes**:
  - Avoid sweeping refactors of untouched or adjacent code.
  - Avoid renaming unrelated functions, structs, or variables.
  - Avoid modifying formatting on unrelated lines.
  - Avoid introducing complex abstractions or indirection when a simple, direct fix suffices.
- Keep the diff tight, focused, and directly addressing the root cause.

### 5. Prune Unused Code Immediately
- If any functions, struct fields, methods, or helpers become unused as a result of your changes, remove them immediately.
- Do not keep dead code around for hypothetical future use; never use `#[allow(dead_code)]`.
- **No backwards compatibility requirement**: There is no need to preserve backwards compatibility for internal code or superseded interfaces unless the user explicitly requests it.

---

## Workflow Step-by-Step

1. **Investigate & Isolate**:
   - Trace the reported issue to the responsible crate or subsystem.
   - Determine whether a unit test (`path/to/foo_test.rs`) or an integration test (`ui/tests/...` or crate integration test) is simpler.

2. **Write the Failing Test**:
   - Follow the standard 3-section test layout (setup -> action -> assertions).
   - Ensure the test is annotated with `#[tracing::instrument(level = "debug", skip_all)]` and uses `rstest`.
   - Run the test using Bazel (e.g. `bazel test //path/to/crate:test`) to confirm it fails with the expected symptom.

3. **Implement the Minimal Fix**:
   - Locate the root cause and implement the smallest necessary change.
   - Remove any newly orphaned functions, variables, or types.

4. **Verify Without Modifying Tests**:
   - Re-run the reproduction test and ensure it now passes without any changes to the test itself.
   - Run the full crate test suite to verify no regressions in existing tests.
   - Validate instrumentation: `bazel run //tools:check_instrument`.
   - Format changed files: `bazel run //tools:format -- <files>`.
