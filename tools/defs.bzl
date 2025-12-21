def _expand_template_impl(ctx):
    output = ctx.actions.declare_file(ctx.attr.out_name)

    # --- CHANGE 1: Expand Locations ---
    # We iterate over the input dictionary and resolve $(execpath) tags.
    # ctx.expand_location(string, targets) handles the magic.
    final_subs = {}
    for key, value in ctx.attr.substitutions.items():
        # data needs to be passed here so Bazel can look up the targets
        expanded_val = ctx.expand_location(value, ctx.attr.data)
        final_subs[key] = expanded_val

    # (Optional) processing for target_substitutions from previous steps...
    if hasattr(ctx.attr, "target_substitutions"):
        for target, placeholder in ctx.attr.target_substitutions.items():
            files = target.files.to_list()
            if len(files) != 1:
                fail("Target {} must produce exactly one file".format(target.label))
            final_subs[placeholder] = files[0].basename

    # Encode the expanded dictionary to JSON
    subs_json = json.encode(final_subs)

    args = ctx.actions.args()
    args.add(ctx.file.template.path)
    args.add(output.path)
    args.add(subs_json)

    ctx.actions.run(
        outputs = [output],
        # --- CHANGE 2: Add data to inputs ---
        # We must add ctx.files.data to inputs, or Bazel won't build them
        # before running this action (though for simple expansion it matters less,
        # it is required for correctness).
        inputs = [ctx.file.template] + ctx.files.data,
        executable = ctx.executable._builder,
        tools = [ctx.executable._builder],
        arguments = [args],
        mnemonic = "ExpandTemplate",
        progress_message = "Expanding template {}...".format(ctx.attr.out_name),
    )

    return [DefaultInfo(files = depset([output]))]

expand_template = rule(
    implementation = _expand_template_impl,
    attrs = {
        "template": attr.label(allow_single_file = True, mandatory = True),
        "out_name": attr.string(default = "index.html"),
        "substitutions": attr.string_dict(),
        # --- CHANGE 3: New Attribute ---
        # This allows you to pass the targets referenced in substitutions
        "data": attr.label_list(allow_files = True),
        "target_substitutions": attr.label_keyed_string_dict(allow_files = True),
        "_builder": attr.label(
            default = Label("//tools:replace_bin"),
            executable = True,
            cfg = "exec",
        ),
    },
)

def _assemble_moosync_dist_impl(ctx):
    output_dir = ctx.actions.declare_directory(ctx.attr.out_dir_name)

    args = ctx.actions.args()
    args.add(output_dir.path)
    args.add_all(ctx.files.srcs)

    ctx.actions.run(
        outputs = [output_dir],
        inputs = ctx.files.srcs,
        executable = ctx.executable._builder,
        tools = [ctx.executable._builder],
        arguments = [args],
        mnemonic = "AssembleDist",
        progress_message = "Assembling dist directory...",
    )

    return [DefaultInfo(files = depset([output_dir]))]

assemble_moosync_dist = rule(
    implementation = _assemble_moosync_dist_impl,
    doc = """
    Collects a list of files and directories and copies them into a single output directory.
    
    This is useful for creating a 'dist' folder for web deployment. It flattens
    the directory structure (all inputs are copied to the root of the output dir).
    """,
    attrs = {
        "srcs": attr.label_list(
            allow_files = True,
            mandatory = True,
            doc = "List of files or targets (like bindgen outputs, HTML files) to copy.",
        ),
        "out_dir_name": attr.string(
            default = "dist",
            doc = "The name of the output directory to create.",
        ),
        "_builder": attr.label(
            default = Label("//tools:assemble_bin"),
            executable = True,
            cfg = "exec",
        ),
    },
)
