load("@rules_cc//cc/common:cc_common.bzl", "cc_common")
load("@rules_cc//cc/common:cc_info.bzl", "CcInfo")

def _expand_template_impl(ctx):
    output = ctx.actions.declare_file(ctx.attr.out_name)

    final_subs = {}
    for key, value in ctx.attr.substitutions.items():
        expanded_val = ctx.expand_location(value, ctx.attr.data)
        final_subs[key] = expanded_val

    if hasattr(ctx.attr, "target_substitutions"):
        for target, placeholder in ctx.attr.target_substitutions.items():
            files = target.files.to_list()
            if len(files) != 1:
                fail("Target {} must produce exactly one file".format(target.label))
            final_subs[placeholder] = files[0].basename

    subs_json = json.encode(final_subs)

    args = ctx.actions.args()
    args.add(ctx.file.template.path)
    args.add(output.path)
    args.add(subs_json)

    ctx.actions.run(
        outputs = [output],
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
        "data": attr.label_list(allow_files = True),
        "target_substitutions": attr.label_keyed_string_dict(allow_files = True),
        "_builder": attr.label(
            default = Label("//tools:replace_bin"),
            executable = True,
            cfg = "exec",
        ),
    },
)

def _html_page_impl(ctx):
    out_dir = ctx.actions.declare_directory(ctx.attr.name)

    expanded_content = ctx.expand_location(ctx.attr.content, targets = ctx.attr.data)

    args = ctx.actions.args()
    args.add("--output_dir", out_dir.path)
    args.add("--title", ctx.attr.title)
    args.add("--header", ctx.attr.header)
    args.add("--content", expanded_content)
    args.add("--public_path", ctx.attr.public_path)

    args.add_all("--css", ctx.files.css)
    args.add_all("--js", ctx.files.js)
    args.add_all("--wasm", ctx.files.wasm)
    args.add_all("--fonts", ctx.files.fonts)
    args.add_all("--assets", ctx.files.assets)

    all_inputs = (
        ctx.files.css +
        ctx.files.js +
        ctx.files.wasm +
        ctx.files.fonts +
        ctx.files.assets +
        ctx.files.data
    )

    ctx.actions.run(
        outputs = [out_dir],
        inputs = all_inputs,
        executable = ctx.executable._generator,
        arguments = [args],
        mnemonic = "HtmlBundle",
    )

    return [DefaultInfo(
        files = depset([out_dir]),
        runfiles = ctx.runfiles(files = [out_dir]),
    )]

html_page = rule(
    implementation = _html_page_impl,
    doc = """
Generates a static HTML bundle directory with `index.html` and an `assets/` folder.
Handles copying and linking CSS, JS, WASM (preload), and Fonts (preload).
""",
    attrs = {
        "title": attr.string(
            mandatory = True,
            doc = "The content of the <title> tag.",
        ),
        "header": attr.string(
            doc = "The main H1 header text displayed at the top of the body.",
        ),
        "content": attr.string(
            doc = "The main text body. Supports $(rootpath) expansion.",
        ),
        "public_path": attr.string(
            default = "",
            doc = "A URL prefix to prepend to asset links.",
        ),
        "css": attr.label_list(
            allow_files = [".css", ".map"],
            doc = "CSS files to link in the <head>.",
        ),
        "js": attr.label_list(
            allow_files = [".js", ".map"],
            doc = "JS files to include at the bottom of the <body>.",
        ),
        "wasm": attr.label_list(
            allow_files = [".wasm"],
            doc = "WASM binaries to be preloaded in <head>.",
        ),
        "fonts": attr.label_list(
            allow_files = [".woff", ".woff2", ".ttf", ".otf"],
            doc = "Font files to be preloaded in <head>.",
        ),
        "assets": attr.label_list(
            allow_files = True,
            doc = "Generic assets (images, etc) to copy to the output directory.",
        ),
        "data": attr.label_list(
            allow_files = True,
            doc = "Targets referenced in the 'content' string for location expansion.",
        ),
        "_generator": attr.label(
            default = Label("//tools:html_generator"),
            executable = True,
            cfg = "exec",
            doc = "The internal Python tool used to generate the bundle.",
        ),
    },
)

def _pkg_config_impl(ctx):
    out = ctx.actions.declare_file("lib/pkgconfig/" + ctx.attr.lib_name + ".pc")
    content = """prefix=${{pcfiledir}}/..
exec_prefix=${{prefix}}
libdir=${{prefix}}/lib/lib
includedir=${{prefix}}/include

Name: {name}
Description: {desc}
Version: {version}
Requires: {requires}
Libs: -L${{libdir}} {libs}
Cflags: -I${{includedir}} {cflags}
""".format(
        name = ctx.attr.lib_name,
        desc = ctx.attr.description,
        version = ctx.attr.version,
        requires = ", ".join(ctx.attr.requires),
        libs = " ".join(["-l" + l for l in ctx.attr.libs]),
        cflags = " ".join(ctx.attr.cflags),
    )

    ctx.actions.write(out, content)

    return [
        DefaultInfo(files = depset([out])),
        CcInfo(
            compilation_context = cc_common.create_compilation_context(
                headers = depset([out]),
            ),
            linking_context = cc_common.create_linking_context(
                linker_inputs = depset([]),
            ),
        ),
    ]

pkg_config = rule(
    implementation = _pkg_config_impl,
    attrs = {
        "lib_name": attr.string(mandatory = True),
        "description": attr.string(default = "Library"),
        "version": attr.string(default = "1.0.0"),
        "requires": attr.string_list(default = []),
        "libs": attr.string_list(mandatory = True),  # e.g. ["ogg"] -> -logg
        "cflags": attr.string_list(default = []),
    },
)

def _dirgroup_impl(ctx):
    output_dir = ctx.actions.declare_directory(ctx.attr.name)
    cmd = "cp -fL {srcs} {out_dir}".format(
        srcs = " ".join([f.path for f in ctx.files.srcs]),
        out_dir = output_dir.path,
    )

    ctx.actions.run_shell(
        inputs = ctx.files.srcs,
        outputs = [output_dir],
        command = cmd,
        mnemonic = "CopyToDir",
    )
    return [DefaultInfo(files = depset([output_dir]))]

dirgroup = rule(
    implementation = _dirgroup_impl,
    attrs = {
        "srcs": attr.label_list(allow_files = True),
    },
)
