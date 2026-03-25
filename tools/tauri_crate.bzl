"""Repository rule to discover the tauri crate workspace name and android files."""

def _extract_tauri_version(lockfile):
    """Extract X.Y.Z version from lockfile content."""
    marker = "crates__tauri-"
    for line in lockfile.split("\n"):
        if marker in line and "crates__tauri-plugin" not in line:
            idx = line.find(marker)
            if idx == -1:
                continue
            remainder = line[idx + len(marker):]
            version_chars = []
            for c in remainder:
                if c.isdigit() or c == ".":
                    version_chars.append(c)
                else:
                    break
            if version_chars:
                return "".join(version_chars)
    return None

def _scan_android_files(rctx, crate_path):
    """Scan the tauri crate directory for mobile/android files."""
    result = rctx.execute([
        "find", crate_path + "/mobile/android",
        "-type", "f",
        "-not", "-path", "*/test/*",
        "-not", "-path", "*/androidTest/*",
    ], quiet = True)

    if result.return_code != 0:
        return None

    files = []
    prefix = crate_path + "/"
    for line in result.stdout.strip().split("\n"):
        if line:
            # Convert absolute path to relative
            if line.startswith(prefix):
                files.append(line[len(prefix):])
            else:
                files.append(line)
    return sorted(files)

def _tauri_crate_repo_impl(rctx):
    # Read MODULE.bazel.lock to find the tauri crate version
    lockfile = rctx.read(rctx.path(Label("//:MODULE.bazel.lock")))

    version = _extract_tauri_version(lockfile)
    if not version:
        fail("Could not find tauri crate version in MODULE.bazel.lock")

    workspace = "rules_rust++_crate+++crates+crates__tauri-" + version

    # Try to find the crate in bazel's external directory
    # rctx.path(".") is inside the repo being created, go up to external/
    repo_root = str(rctx.path(".").dirname)  # This is external/+tauri_extension+tauri_crate
    external_dir = str(rctx.path(".").dirname.dirname)  # This is external/

    external_paths = [
        external_dir + "/" + workspace,
    ]

    android_files = None
    for ext_path in external_paths:
        android_files = _scan_android_files(rctx, ext_path)
        if android_files:
            break

    # Fallback to hardcoded list if scan fails
    if not android_files:
        android_files = [
            "mobile/android/.gitignore",
            "mobile/android/build.gradle.kts",
            "mobile/android/proguard-rules.pro",
            "mobile/android/src/main/AndroidManifest.xml",
            "mobile/android/src/main/java/app/tauri/AppPlugin.kt",
            "mobile/android/src/main/java/app/tauri/FsUtils.kt",
            "mobile/android/src/main/java/app/tauri/JniMethod.kt",
            "mobile/android/src/main/java/app/tauri/Logger.kt",
            "mobile/android/src/main/java/app/tauri/PathPlugin.kt",
            "mobile/android/src/main/java/app/tauri/PermissionHelper.kt",
            "mobile/android/src/main/java/app/tauri/PermissionState.kt",
            "mobile/android/src/main/java/app/tauri/annotation/ActivityCallback.kt",
            "mobile/android/src/main/java/app/tauri/annotation/InvokeArg.kt",
            "mobile/android/src/main/java/app/tauri/annotation/Permission.kt",
            "mobile/android/src/main/java/app/tauri/annotation/PermissionCallback.kt",
            "mobile/android/src/main/java/app/tauri/annotation/PluginMethod.kt",
            "mobile/android/src/main/java/app/tauri/annotation/TauriPlugin.kt",
            "mobile/android/src/main/java/app/tauri/plugin/Channel.kt",
            "mobile/android/src/main/java/app/tauri/plugin/InvalidPluginMethodException.kt",
            "mobile/android/src/main/java/app/tauri/plugin/Invoke.kt",
            "mobile/android/src/main/java/app/tauri/plugin/JSArray.kt",
            "mobile/android/src/main/java/app/tauri/plugin/JSObject.kt",
            "mobile/android/src/main/java/app/tauri/plugin/Plugin.kt",
            "mobile/android/src/main/java/app/tauri/plugin/PluginHandle.kt",
            "mobile/android/src/main/java/app/tauri/plugin/PluginManager.kt",
            "mobile/android/src/main/java/app/tauri/plugin/PluginMethodData.kt",
            "mobile/android/src/main/java/app/tauri/plugin/PluginResult.kt",
        ]

    # Format file list for .bzl
    files_str = ",\n    ".join(['"{}"'.format(f) for f in android_files])

    # Generate a .bzl file with the constants
    rctx.file("BUILD.bazel", "")
    rctx.file("defs.bzl", """
# Auto-generated - do not edit
# Resolved from @crates//:tauri alias (version {version})

TAURI_CRATE = "{workspace}"

TAURI_ANDROID_FILES = [
    {files}
]

def tauri_android_srcs():
    \"\"\"Returns list of tauri android source labels.\"\"\"
    return ["@@{{}}//:{{}}".format(TAURI_CRATE, f) for f in TAURI_ANDROID_FILES]
""".format(workspace = workspace, version = version, files = files_str))

tauri_crate_repo = repository_rule(
    implementation = _tauri_crate_repo_impl,
    local = True,  # Re-run when local files change
)

def _tauri_extension_impl(mctx):
    tauri_crate_repo(name = "tauri_crate")
    return mctx.extension_metadata(
        reproducible = True,
        root_module_direct_deps = ["tauri_crate"],
        root_module_direct_dev_deps = [],
    )

tauri_extension = module_extension(
    implementation = _tauri_extension_impl,
)
