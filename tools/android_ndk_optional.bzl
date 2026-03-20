"""Optional Android NDK repository extension.

This wrapper makes the Android NDK toolchain optional - when the NDK is not
installed, it provides a stub repository with no toolchains instead of failing.
This allows desktop-only builds to proceed without the NDK.

Usage in MODULE.bazel:
    bazel_dep(name = "rules_android_ndk", version = "0.1.3")  # Keep for the repository rule

    android_ndk_optional = use_extension("//tools:android_ndk_optional.bzl", "android_ndk_optional")
    android_ndk_optional.configure(api_level = 26)
    use_repo(android_ndk_optional, "androidndk")

    register_toolchains("@androidndk//:all")
"""

load("@rules_android_ndk//:rules.bzl", "android_ndk_repository")

def _find_ndk_path(rctx):
    """Attempt to find the Android NDK path from environment variables."""
    # Check ANDROID_NDK_HOME first (explicit NDK path)
    ndk_home = rctx.os.environ.get("ANDROID_NDK_HOME", "")
    if ndk_home:
        ndk_path = rctx.path(ndk_home)
        if ndk_path.exists:
            return ndk_home

    # Check ANDROID_HOME/ndk (SDK with NDK installed)
    android_home = rctx.os.environ.get("ANDROID_HOME", "")
    if android_home:
        ndk_base = rctx.path(android_home + "/ndk")
        if ndk_base.exists:
            # Find the highest version NDK installed
            result = rctx.execute(["ls", "-1", str(ndk_base)], quiet = True)
            if result.return_code == 0:
                versions = [v.strip() for v in result.stdout.strip().split("\n") if v.strip()]
                if versions:
                    versions.sort()
                    return android_home + "/ndk/" + versions[-1]

    return None

def _stub_androidndk_impl(rctx):
    """Create a stub @androidndk repository with no toolchains."""
    # Empty BUILD file - no toolchains provided
    # register_toolchains("@androidndk//:all") will expand to nothing (no toolchain rules)
    # This allows desktop builds to proceed without NDK
    rctx.file("BUILD.bazel", """\
# Stub Android NDK repository - NDK not available
# This allows builds to proceed without Android NDK installed
# No toolchain() rules = register_toolchains(":all") registers nothing

package(default_visibility = ["//visibility:public"])

# Stub for --android_crosstool_top=@androidndk//:toolchain
# This will error if actually used for Android builds, but allows
# analysis to proceed for non-Android targets
filegroup(
    name = "toolchain",
    srcs = [],
    tags = ["manual"],
)
""")

_stub_androidndk = repository_rule(
    implementation = _stub_androidndk_impl,
    local = True,
    environ = ["ANDROID_NDK_HOME", "ANDROID_HOME"],
    doc = "Creates a stub Android NDK repository when NDK is not available",
)

# Tag class for configuring the extension
_configure = tag_class(
    attrs = {
        "api_level": attr.int(default = 21, doc = "Android API level for NDK"),
    },
)

def _android_ndk_optional_impl(module_ctx):
    """Module extension that conditionally provides Android NDK toolchains."""

    # Get configuration from tags
    api_level = 21
    for mod in module_ctx.modules:
        for tag in mod.tags.configure:
            api_level = tag.api_level

    # Check if NDK is available
    ndk_home = module_ctx.os.environ.get("ANDROID_NDK_HOME", "")
    android_home = module_ctx.os.environ.get("ANDROID_HOME", "")

    ndk_available = False
    ndk_path = None

    if ndk_home:
        ndk_available = True
        ndk_path = ndk_home
    elif android_home:
        # Check for NDK under ANDROID_HOME
        # We can't do path existence checks in module extension context,
        # but we can check in the repository rule
        ndk_available = True
        ndk_path = None  # Let repository rule figure it out

    if ndk_available:
        # Use the real android_ndk_repository from rules_android_ndk
        android_ndk_repository(
            name = "androidndk",
            api_level = api_level,
        )
    else:
        # Create stub repository
        _stub_androidndk(name = "androidndk")

    return module_ctx.extension_metadata(
        reproducible = False,  # Depends on environment
        root_module_direct_deps = ["androidndk"],
        root_module_direct_dev_deps = [],
    )

android_ndk_optional = module_extension(
    implementation = _android_ndk_optional_impl,
    tag_classes = {
        "configure": _configure,
    },
    os_dependent = True,
    environ = ["ANDROID_NDK_HOME", "ANDROID_HOME"],
    doc = "Conditionally provides Android NDK toolchains, with stub fallback when NDK unavailable",
)
