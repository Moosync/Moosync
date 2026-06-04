load("@rules_cc//cc/common:cc_common.bzl", "cc_common")
load("@rules_cc//cc/toolchains:cc_toolchain_config_info.bzl", "CcToolchainConfigInfo")
load("@rules_cc//cc:cc_toolchain_config_lib.bzl", "tool_path")
load("@rules_rust//rust:defs.bzl", "rust_binary")

def _flutter_app_bundle_impl(ctx):
    out_dir = ctx.actions.declare_directory(ctx.attr.name)
    
    # Find the tree artifact containing the build bundle
    tree_artifact = None
    for f in ctx.files.app:
        if f.is_directory:
            tree_artifact = f
            break
            
    if not tree_artifact:
        fail("Could not find build artifacts directory in app")
        
    rust_lib = ctx.file.moosync_rust
    platform = ctx.attr.platform
    
    if platform == "linux":
        script = """
        rm -rf "{out}"
        mkdir -p "{out}"
        cp -RL "{src_dir}"/* "{out}/"
        mkdir -p "{out}/lib"
        cp -f "{rust_lib}" "{out}/lib/"
        """.format(
            out = out_dir.path,
            src_dir = tree_artifact.path,
            rust_lib = rust_lib.path,
        )
    elif platform == "macos":
        script = """
        rm -rf "{out}"
        mkdir -p "{out}"
        cp -RL "{src_dir}"/* "{out}/"
        app_dir=$(find "{out}" -maxdepth 2 -name "*.app" | head -n 1)
        if [ -n "$app_dir" ]; then
            mkdir -p "$app_dir/Contents/Frameworks"
            cp -f "{rust_lib}" "$app_dir/Contents/Frameworks/"
        else
            echo "Error: Could not find .app directory in macOS build" >&2
            exit 1
        fi
        """.format(
            out = out_dir.path,
            src_dir = tree_artifact.path,
            rust_lib = rust_lib.path,
        )
    elif platform == "windows":
        script = """
        rm -rf "{out}"
        mkdir -p "{out}"
        cp -RL "{src_dir}"/* "{out}/"
        cp -f "{rust_lib}" "{out}/"
        """.format(
            out = out_dir.path,
            src_dir = tree_artifact.path,
            rust_lib = rust_lib.path,
        )
    else:
        fail("Unsupported platform for bundling: " + platform)
        
    ctx.actions.run_shell(
        inputs = [tree_artifact, rust_lib],
        outputs = [out_dir],
        command = script,
        mnemonic = "BundleFlutterApp",
        progress_message = "Bundling Flutter App with Rust library for %s (%s)" % (ctx.label.name, platform),
    )
    
    return [DefaultInfo(files = depset([out_dir]))]

flutter_app_bundle = rule(
    implementation = _flutter_app_bundle_impl,
    attrs = {
        "app": attr.label(mandatory = True),
        "moosync_rust": attr.label(mandatory = True, allow_single_file = True),
        "platform": attr.string(mandatory = True, values = ["linux", "macos", "windows"]),
    },
)

def _platform_transition_impl(settings, attr):
    return {"//command_line_option:platforms": [str(attr.platform)]}

platform_transition = transition(
    implementation = _platform_transition_impl,
    inputs = [],
    outputs = ["//command_line_option:platforms"],
)

def _platform_transition_rule_impl(ctx):
    return [DefaultInfo(files = depset(ctx.files.src))]

platform_transition_rule = rule(
    implementation = _platform_transition_rule_impl,
    attrs = {
        "src": attr.label(mandatory = True),
        "platform": attr.label(mandatory = True),
        "_allowlist_function_transition": attr.label(
            default = "@bazel_tools//tools/allowlists/function_transition_allowlist",
        ),
    },
    cfg = platform_transition,
)

def rust_benchmark(name, **kwargs):
    rust_binary(
        name = name,
        **kwargs
    )



