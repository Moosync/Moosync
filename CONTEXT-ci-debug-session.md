# CI Debug Session Context

*March 2026 - Debugging GitHub Actions build workflow*

## Problem

Android x86_64 build was getting OOM killed during final Rust cdylib linking step. GHA ubuntu-22.04 runners have 16GB RAM (public repos), but the linker was exceeding that.

## Fix 1: Add Swap Space

Added `pierotofy/set-swap-space@master` action after `free-disk-space` step:

```yaml
- name: Set up swap space
  if: matrix.os == 'ubuntu-22.04'
  uses: pierotofy/set-swap-space@master
  with:
    swap-size-gb: 10
```

Disk space available after `free-disk-space`: ~22GB realistic (7.6GB saved + 14GB baseline). 10GB swap leaves room for Bazel cache (~8-12GB for 3 builds).

## Fix 2: setup-bazel Fork for Iterative Caching

**Problem**: Bazel cache via `bazel-contrib/setup-bazel` doesn't update on cache hits. If you restore a cache and add new artifacts during build, they're lost.

**Our fork**: `MulverineX/setup-bazel@replace-cache-after-change`

Changes:
1. Hash cache contents after restore (using filename list - Bazel uses CAS so filenames are content hashes)
2. After build, compare hashes
3. If changed: delete old cache via GitHub API, save new cache
4. Requires `actions: write` permission

Also migrated from ncc to esbuild because ncc can't handle ESM-only @actions packages.

**Workflow changes**:
```yaml
permissions:
  actions: write
  contents: read

# ...

- uses: MulverineX/setup-bazel@replace-cache-after-change
```

## Current State

- Fork committed and pushed: https://github.com/MulverineX/setup-bazel/tree/replace-cache-after-change
- Workflow updated in Moosync but changes not yet pushed to Moosync repo
- CI run 23408280177 was testing the swap fix (old setup-bazel version)

## Files Modified

- `.github/workflows/build.yaml` - Added swap, permissions, fork reference
- `.github/setup-bazel/` - Cloned fork (gitignored)

## Build Timing Reference

From run 23408280177:
- Desktop build (Bazel Build Package): ~48 min (cold cache)
- Android APKs (both archs): ~23 min (benefits from shared artifacts)
- ARM64 desktop with warm cache: ~2.5 min

## Related Discussion

Discussed Rust ecosystem's lack of binary distribution for proc-macros/build scripts, contributing to slow CI times. See Obsidian note: `Notes/Rust Binary Distribution Problem.md`

## Next Steps

1. Push Moosync workflow changes
2. Verify setup-bazel fork works with iterative caching
3. Monitor if swap resolves OOM on x86_64 Android build
