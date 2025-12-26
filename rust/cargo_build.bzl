def _cargo_generic_build_impl(ctx):
    source_dir = ctx.files.srcs[0]
    out_name = ctx.attr.out_name
    outputs = []

    if ctx.attr.run_wasm_bindgen:
        out_wasm = ctx.actions.declare_file(out_name + "_bg.wasm")
        out_js = ctx.actions.declare_file(out_name + ".js")
        outputs.extend([out_wasm, out_js])
    else:
        out_bin = ctx.actions.declare_file(out_name)
        outputs.append(out_bin)
        out_wasm = out_bin
        out_js = None

    expanded_env = {}
    for k, v in ctx.attr.env.items():
        expanded_env[k] = ctx.expand_location(v, targets = ctx.attr.data)

    env_script = "\n".join([
        "export {}=\"{}\"".format(k, v)
        for k, v in expanded_env.items()
    ])

    target_val = ctx.attr.target
    target_subdir = target_val if target_val else "."
    target_flag = "--target " + target_val if target_val else ""

    comp_mode = ctx.var.get("COMPILATION_MODE", "fastbuild")
    is_release = comp_mode == "opt"
    cargo_mode_flag = "--release" if is_release else ""

    if ctx.attr.run_wasm_bindgen:
        if ctx.attr.wasm_bindgen_path:
            bindgen_bin_path = ctx.attr.wasm_bindgen_path
        else:
            bindgen_bin_path = ctx.executable._wasm_bindgen_hermetic.path
    else:
        bindgen_bin_path = ""

    script = """
    set -e
    ROOT=$(pwd)

    BUILD_DIR=bazel-out/b/build_dir
    
    mkdir -p $BUILD_DIR
    cp -R -L {source_path}/* $BUILD_DIR/
    chmod -R +w $BUILD_DIR
    
    {env_vars}

    cd $BUILD_DIR

    export RUSTFLAGS="{rustflags}"
    
    echo "--- Starting Cargo Build ---"
    cargo build {target_flag} {cargo_mode_flag} {cargo_args}
    
    SEARCH_BASE="target/{target_subdir}"
    
    RAW_BIN=$(find $SEARCH_BASE -maxdepth 3 -type f -name "*.wasm" | head -n 1)

    if [ -z "$RAW_BIN" ]; then
        RAW_BIN=$(find $SEARCH_BASE/debug $SEARCH_BASE/release -maxdepth 1 -type f -not -name "*.*" -executable 2>/dev/null | head -n 1)
    fi

    if [ -z "$RAW_BIN" ]; then
        echo "ERROR: Could not find build artifact in $SEARCH_BASE"
        exit 1
    fi

    if [ "{run_bindgen}" = "True" ]; then
        BINDGEN_CMD="{bindgen_bin_path}"
        if [[ ! "$BINDGEN_CMD" == /* ]] && [[ -f "$ROOT/$BINDGEN_CMD" ]]; then
            BINDGEN_CMD="$ROOT/$BINDGEN_CMD"
        fi

        echo "--- Running Wasm-Bindgen using: $BINDGEN_CMD ---"
        $BINDGEN_CMD "$RAW_BIN" --target web --out-dir . --out-name "{out_name}" --no-typescript
        cp "{out_name}_bg.wasm" "$ROOT/{out_wasm_path}"
        cp "{out_name}.js" "$ROOT/{out_js_path}"
    else
        cp "$RAW_BIN" "$ROOT/{out_wasm_path}"
    fi
    """.format(
        source_path = source_dir.path,
        env_vars = env_script,
        rustflags = " ".join(ctx.attr.rustflags),
        target = target_val,
        target_flag = target_flag,
        target_subdir = target_subdir,
        cargo_args = ctx.attr.cargo_args,
        run_bindgen = ctx.attr.run_wasm_bindgen,
        bindgen_bin_path = bindgen_bin_path,
        out_name = out_name,
        out_wasm_path = out_wasm.path,
        out_js_path = out_js.path if out_js else "",
        cargo_mode_flag = cargo_mode_flag,
    )

    all_inputs = ctx.files.srcs + ctx.files.data
    if ctx.attr.run_wasm_bindgen and not ctx.attr.wasm_bindgen_path:
        all_inputs.append(ctx.executable._wasm_bindgen_hermetic)

    ctx.actions.run_shell(
        inputs = all_inputs,
        outputs = outputs,
        command = script,
        mnemonic = "CargoGenericBuild",
        use_default_shell_env = True,
        execution_requirements = {
            "requires-network": "",
            "no-sandbox": "1",
        },
    )

    return [DefaultInfo(files = depset(outputs))]

cargo_build_ = rule(
    implementation = _cargo_generic_build_impl,
    attrs = {
        "srcs": attr.label_list(allow_files = True, mandatory = True),
        "data": attr.label_list(allow_files = True),
        "target": attr.string(default = ""),
        "rustflags": attr.string_list(default = []),
        "env": attr.string_dict(default = {}),
        "cargo_args": attr.string(default = ""),
        "run_wasm_bindgen": attr.bool(default = False),
        "out_name": attr.string(mandatory = True),
        "wasm_bindgen_path": attr.string(default = ""),
        "_wasm_bindgen_hermetic": attr.label(
            default = Label("@bindeps//:wasm-bindgen-cli__wasm-bindgen"),
            cfg = "exec",
            executable = True,
            allow_files = True,
        ),
    },
)
