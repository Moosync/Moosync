load("@rules_rust//rust/private:providers.bzl", "CrateInfo")

CRATE_NAME_OVERRIDE = {
    "crypto": "rust-crypto",
}

GIT_OVERRIDES = {
    "tauri_plugin_autostart": struct(repo = "https://github.com/tauri-apps/plugins-workspace", branch = "v2"),
    "tauri_plugin_deep-link": struct(repo = "https://github.com/tauri-apps/plugins-workspace", branch = "v2"),
    "tauri_plugin_dialog": struct(repo = "https://github.com/tauri-apps/plugins-workspace", branch = "v2"),
    "tauri_plugin_opener": struct(repo = "https://github.com/tauri-apps/plugins-workspace", branch = "v2"),
    "tauri_plugin_single-instance": struct(repo = "https://github.com/tauri-apps/plugins-workspace", branch = "v2"),
    "tauri_plugin_updater": struct(repo = "https://github.com/tauri-apps/plugins-workspace", branch = "v2"),
    "diesel_logger": struct(repo = "https://github.com/Ovenoboyo/diesel-logger.git", branch = "master"),
}

KNOWN_CRATE_RENAMES = [
    "serde_json",
    "serde_yaml",
    "once_cell",
    "lazy_static",
    "ring",
    "console_error_panic_hook",
    "indexed_db_futures",
    "leptos_context_menu",
    "leptos_virtual_scroller",
    "leptos_dom",
    "leptos_i18n",
    "leptos_meta",
    "leptos_router",
    "leptos_i18n_build",
    "diesel_logger",
    "diesel_migrations",
    "fs_extra",
    "fast_image_resize",
    "num_cpus",
    "env_logger",
    "json_dotpath",
    "rusty_ffmpeg",
]

CargoGenInfo = provider(
    fields = {
        "package_info": "Struct containing package metadata",
        "transitive_packages": "Depset of package info structs",
        "is_build_script_target": "Boolean indicating if this is a build script",
        "build_dependencies": "Tuple of build dependency structs",
        "build_script_source_file": "File or None",
        "build_script_data_files": "Tuple of Files",
    },
)

def _determine_cargo_package_name(bazel_target_name):
    if bazel_target_name in CRATE_NAME_OVERRIDE:
        return CRATE_NAME_OVERRIDE[bazel_target_name]
    if bazel_target_name in KNOWN_CRATE_RENAMES:
        return bazel_target_name
    return bazel_target_name.replace("_", "-")

def _extract_files_from_list(file_list_or_target):
    extracted_files = []
    cargo_toml_file = None

    if type(file_list_or_target) == "list":
        for target in file_list_or_target:
            target_files = target.files.to_list()
            extracted_files.extend(target_files)
            for file in target_files:
                if file.basename == "Cargo.toml":
                    cargo_toml_file = file
    elif file_list_or_target:
        target_files = file_list_or_target.files.to_list()
        extracted_files.extend(target_files)
        for file in target_files:
            if file.basename == "Cargo.toml":
                cargo_toml_file = file
    return extracted_files, cargo_toml_file

def _collect_source_files_and_manifest(context):
    source_files = []
    extra_cargo_toml_file = None

    srcs_files, srcs_toml = _extract_files_from_list(getattr(context.rule.attr, "srcs", []))
    source_files.extend(srcs_files)
    if srcs_toml:
        extra_cargo_toml_file = srcs_toml

    compile_data_files, compile_data_toml = _extract_files_from_list(getattr(context.rule.attr, "compile_data", []))
    if compile_data_toml:
        extra_cargo_toml_file = compile_data_toml

    for file in compile_data_files:
        if file.basename != "Cargo.toml":
            source_files.append(file)

    _, data_toml = _extract_files_from_list(getattr(context.rule.attr, "data", []))
    if data_toml and not extra_cargo_toml_file:
        extra_cargo_toml_file = data_toml

    return source_files, extra_cargo_toml_file

def _determine_crate_root_file(context, source_files):
    crate_root_attribute_files, _ = _extract_files_from_list(getattr(context.rule.attr, "crate_root", []))
    if crate_root_attribute_files:
        crate_root_file = crate_root_attribute_files[0]
        if crate_root_file not in source_files:
            source_files.insert(0, crate_root_file)
        return crate_root_file, source_files

    if not source_files:
        return None, source_files

    is_binary_target = context.rule.kind == "rust_binary"
    target_name = context.label.name
    potential_root_files = []

    for file in source_files:
        if is_binary_target and (file.basename == "main.rs" or file.basename == target_name + ".rs"):
            potential_root_files.insert(0, file)
        elif not is_binary_target and (file.basename == "lib.rs" or file.basename == target_name + ".rs"):
            potential_root_files.insert(0, file)
        else:
            potential_root_files.append(file)

    return potential_root_files[0] if potential_root_files else source_files[0], source_files

def _extract_build_script_info(context):
    build_script_source = None
    build_script_data = []

    if hasattr(context.rule.attr, "script"):
        script_target = context.rule.attr.script
        if CargoGenInfo in script_target:
            provider_info = script_target[CargoGenInfo]
            if provider_info.build_script_source_file:
                build_script_source = provider_info.build_script_source_file
            elif provider_info.package_info.crate_root:
                build_script_source = provider_info.package_info.crate_root
            elif len(provider_info.package_info.source_files) > 0:
                build_script_source = provider_info.package_info.source_files[0]

            if hasattr(provider_info, "build_script_data_files"):
                build_script_data.extend(list(provider_info.build_script_data_files))

    if hasattr(context.rule.attr, "data"):
        data_files, _ = _extract_files_from_list(context.rule.attr.data)
        build_script_data.extend(data_files)

    return build_script_source, build_script_data

def _determine_target_type(context):
    kind = context.rule.kind
    is_binary = kind == "rust_binary"
    is_proc_macro = kind == "rust_proc_macro"
    is_runner = kind == "cargo_build_script"
    is_wrapper = hasattr(context.rule.attr, "script") and not is_runner
    is_build_script = is_runner or is_wrapper or hasattr(context.rule.attr, "build_script_env")

    if is_binary and (context.label.name.endswith("_bs") or "_build_script" in context.label.name):
        is_build_script = True

    return is_binary, is_build_script, is_proc_macro

def _create_external_package_info(target, name, package_name, version, features):
    package_info = struct(
        name = name,
        cargo_pkg_name = package_name,
        version = version,
        features = tuple(features),
        is_external = True,
        is_binary = False,
        is_proc_macro = False,
        package_path = target.label.package,
        source_package_path = target.label.package,
        deps = (),
        source_files = (),
        edition = "2024",
        crate_root = None,
        build_script_source_file = None,
        build_script_data_files = (),
        build_dependencies = (),
        extra_cargo_toml = None,
    )
    return [CargoGenInfo(
        package_info = package_info,
        transitive_packages = depset([package_info]),
        is_build_script_target = False,
        build_dependencies = (),
        build_script_source_file = None,
        build_script_data_files = (),
    )]

def _process_dependencies(context):
    direct_dependencies = []
    build_dependencies = []
    transitive_packages = []

    inherited_build_script_source = None
    inherited_build_script_data = []

    all_dependencies = getattr(context.rule.attr, "deps", [])
    if hasattr(context.rule.attr, "proc_macro_deps"):
        all_dependencies = all_dependencies + context.rule.attr.proc_macro_deps

    for dependency in all_dependencies:
        if CargoGenInfo not in dependency:
            continue

        dependency_info = dependency[CargoGenInfo]
        transitive_packages.append(dependency_info.transitive_packages)

        git_override_info = GIT_OVERRIDES.get(dependency_info.package_info.name)

        dependency_data = struct(
            name = dependency_info.package_info.name,
            cargo_pkg_name = dependency_info.package_info.cargo_pkg_name,
            is_external = dependency_info.package_info.is_external,
            version = dependency_info.package_info.version,
            features = dependency_info.package_info.features,
            path = getattr(dependency_info.package_info, "package_path", None),
            git = git_override_info,
        )

        if dependency_info.is_build_script_target:
            if hasattr(dependency_info.package_info, "deps"):
                build_dependencies.extend(dependency_info.package_info.deps)
            inherited_build_script_source = dependency_info.build_script_source_file
            inherited_build_script_data.extend(list(dependency_info.build_script_data_files))
        else:
            direct_dependencies.append(dependency_data)

    return direct_dependencies, build_dependencies, transitive_packages, inherited_build_script_source, inherited_build_script_data

def _process_data_dependencies(context, transitive_packages, source_files):
    if hasattr(context.rule.attr, "data"):
        for data_dependency in context.rule.attr.data:
            if CargoGenInfo in data_dependency:
                transitive_packages.append(data_dependency[CargoGenInfo].transitive_packages)
            else:
                source_files.extend(data_dependency.files.to_list())
    return transitive_packages, source_files

def _finalize_build_script_details(is_build_script, attribute_source, crate_root, source_files, attribute_data, dependency_source, dependency_data):
    final_source_file = None
    final_data_files = []

    if is_build_script:
        if attribute_source:
            final_source_file = attribute_source
        elif crate_root:
            final_source_file = crate_root
        elif len(source_files) > 0:
            final_source_file = source_files[0]
        final_data_files = attribute_data
    else:
        final_source_file = dependency_source
        final_data_files = dependency_data

    return final_source_file, tuple(final_data_files)

def _create_internal_package_info(context, name, package_name, version, features, is_binary, is_build_script, is_proc_macro, dependencies, build_dependencies, source_files, edition, crate_root, build_script_source, build_script_data, extra_cargo_toml):
    package_path = context.label.package
    if is_binary and not is_build_script:
        export_package_path = package_path + "_bin"
    else:
        export_package_path = package_path

    return struct(
        name = name,
        cargo_pkg_name = package_name,
        version = version,
        features = tuple(features),
        is_external = False,
        is_binary = is_binary and not is_build_script,
        is_proc_macro = is_proc_macro,
        package_path = export_package_path,
        source_package_path = package_path,
        deps = tuple(dependencies),
        build_deps = tuple(build_dependencies),
        source_files = tuple(source_files),
        edition = edition,
        crate_root = crate_root,
        build_script_source_file = build_script_source,
        build_script_data_files = build_script_data,
        extra_cargo_toml = extra_cargo_toml,
    )

def _cargo_toml_aspect_impl(target, context):
    source_files, extra_cargo_toml = _collect_source_files_and_manifest(context)
    crate_root_file, source_files = _determine_crate_root_file(context, source_files)

    attribute_bs_source, attribute_bs_data = _extract_build_script_info(context)
    is_binary, is_build_script, is_proc_macro = _determine_target_type(context)

    if is_build_script and hasattr(context.rule.attr, "data"):
        data_files, _ = _extract_files_from_list(context.rule.attr.data)
        attribute_bs_data.extend(data_files)

    is_external_workspace = target.label.workspace_name != "" and target.label.workspace_name != context.workspace_name

    target_name = context.label.name
    if CrateInfo in target:
        target_name = target[CrateInfo].name

    version = getattr(context.rule.attr, "version", "*")
    features = getattr(context.rule.attr, "crate_features", [])
    cargo_package_name = _determine_cargo_package_name(target_name)

    if is_external_workspace:
        return _create_external_package_info(target, target_name, cargo_package_name, version, features)

    if not (context.rule.kind.startswith("rust_") or is_build_script):
        return []

    direct_dependencies, build_dependencies, transitive_packages, dep_bs_source, dep_bs_data = _process_dependencies(context)
    transitive_packages, source_files = _process_data_dependencies(context, transitive_packages, source_files)

    final_bs_source, final_bs_data = _finalize_build_script_details(is_build_script, attribute_bs_source, crate_root_file, source_files, attribute_bs_data, dep_bs_source, dep_bs_data)

    edition = getattr(context.rule.attr, "edition", "2021") or "2021"

    package_info = _create_internal_package_info(
        context,
        target_name,
        cargo_package_name,
        "0.1.0",
        features,
        is_binary,
        is_build_script,
        is_proc_macro,
        direct_dependencies,
        build_dependencies,
        source_files,
        edition,
        crate_root_file,
        final_bs_source,
        final_bs_data,
        extra_cargo_toml,
    )

    direct_packages_to_export = [package_info] if not is_build_script else []

    return [CargoGenInfo(
        package_info = package_info,
        transitive_packages = depset(direct = direct_packages_to_export, transitive = transitive_packages),
        is_build_script_target = is_build_script,
        build_script_source_file = final_bs_source,
        build_script_data_files = final_bs_data,
        build_dependencies = (),
    )]

cargo_aspect = aspect(
    implementation = _cargo_toml_aspect_impl,
    attr_aspects = ["deps", "proc_macro_deps", "data", "compile_data", "script"],
)

def _format_git_dependency_string(dependency):
    spec = 'git = "{}"'.format(dependency.git.repo)
    if hasattr(dependency.git, "branch"):
        spec += ', branch = "{}"'.format(dependency.git.branch)
    elif hasattr(dependency.git, "tag"):
        spec += ', tag = "{}"'.format(dependency.git.tag)
    elif hasattr(dependency.git, "rev"):
        spec += ', rev = "{}"'.format(dependency.git.rev)
    return "{} = {{ {} }}".format(dependency.cargo_pkg_name, spec)

def _format_external_crate_dependency_string(dependency):
    features = ['"{}"'.format(f) for f in dependency.features if f != "default"]
    has_default = "default" in dependency.features
    default_features_bool = "true" if has_default else "false"

    if not features:
        if default_features_bool == "true":
            return '{} = "{}"'.format(dependency.cargo_pkg_name, dependency.version)
        else:
            return '{} = {{ version = "{}", default-features = false }}'.format(dependency.cargo_pkg_name, dependency.version)
    else:
        return '{} = {{ version = "{}", default-features = {}, features = [{}] }}'.format(
            dependency.cargo_pkg_name,
            dependency.version,
            default_features_bool,
            ", ".join(features),
        )

def _format_path_dependency_string(package, dependency):
    clean_path = dependency.path if dependency.path else "."
    package_depth = len(package.package_path.split("/")) if package.package_path else 0
    path_prefix = "../" * package_depth
    return '{} = {{ path = "{}{}" }}'.format(dependency.cargo_pkg_name, path_prefix, clean_path)

def _generate_dependency_section_lines(header, dependencies, package):
    if not dependencies:
        return []
    lines = [header]
    for dependency in dependencies:
        if dependency.git:
            lines.append(_format_git_dependency_string(dependency))
        elif dependency.is_external:
            lines.append(_format_external_crate_dependency_string(dependency))
        else:
            lines.append(_format_path_dependency_string(package, dependency))
    lines.append("")
    return lines

def _generate_target_section_lines(package):
    lines = []
    if package.is_binary and package.crate_root:
        lines.append('[[bin]]\nname = "{}"'.format(package.cargo_pkg_name))
        relative_path = package.crate_root.short_path
        source_dir = getattr(package, "source_package_path", package.package_path)
        if relative_path.startswith(source_dir + "/"):
            relative_path = relative_path[len(source_dir) + 1:]
        lines.append('path = "{}"'.format(relative_path))
    elif not package.is_binary and package.crate_root:
        lines.append("[lib]")
        if getattr(package, "is_proc_macro", False):
            lines.append("proc-macro = true")
        relative_path = package.crate_root.short_path
        source_dir = getattr(package, "source_package_path", package.package_path)
        if relative_path.startswith(source_dir + "/"):
            relative_path = relative_path[len(source_dir) + 1:]
        lines.append('path = "{}"'.format(relative_path))
    return lines

def _generate_cargo_toml_content_lines(package):
    lines = ["[package]"]
    lines.append('name = "{}"'.format(package.cargo_pkg_name))
    lines.append('version = "{}"'.format(package.version))
    lines.append('edition = "{}"'.format(package.edition))
    if package.build_script_source_file:
        lines.append('build = "build.rs"')
    lines.append("")

    lines.extend(_generate_dependency_section_lines("[dependencies]", package.deps, package))
    if hasattr(package, "build_deps"):
        lines.extend(_generate_dependency_section_lines("[build-dependencies]", package.build_deps, package))

    lines.extend(_generate_target_section_lines(package))
    return lines

def _create_merged_cargo_toml_artifact(context, package, toml_content_lines):
    final_artifact = context.actions.declare_file("intermediate_manifests/" + package.package_path + "/Cargo.toml")
    if package.extra_cargo_toml:
        generated_artifact = context.actions.declare_file("intermediate_manifests/" + package.package_path + "/Cargo_generated.toml")
        context.actions.write(generated_artifact, "\n".join(toml_content_lines))
        context.actions.run_shell(
            inputs = [generated_artifact, package.extra_cargo_toml],
            outputs = [final_artifact],
            command = "cat '{gen}' > '{out}' && echo >> '{out}' && cat '{user}' >> '{out}'".format(
                gen = generated_artifact.path,
                user = package.extra_cargo_toml.path,
                out = final_artifact.path,
            ),
            mnemonic = "MergeCargoToml",
        )
    else:
        context.actions.write(final_artifact, "\n".join(toml_content_lines))
    return final_artifact

def _generate_copy_toml_command(package, output_dir, toml_artifact):
    destination_path = output_dir.path + "/" + package.package_path + "/Cargo.toml"
    return "mkdir -p $(dirname {})\ncp -fL {} {}".format(destination_path, toml_artifact.path, destination_path)

def _generate_copy_source_files_commands(package, output_dir):
    shell_commands = []
    action_inputs = []
    source_dir = getattr(package, "source_package_path", package.package_path)

    for source_file in package.source_files:
        if source_file.basename == "Cargo.toml":
            continue
        relative_path = source_file.short_path
        if relative_path.startswith(source_dir + "/"):
            relative_path = relative_path[len(source_dir) + 1:]
        destination_path = output_dir.path + "/" + package.package_path + "/" + relative_path

        action_inputs.append(source_file)
        shell_commands.append("mkdir -p $(dirname {})\ncp -rfL {} {}".format(destination_path, source_file.path, destination_path))
    return action_inputs, shell_commands

def _generate_copy_build_script_commands(package, output_dir):
    shell_commands = []
    action_inputs = []
    if package.build_script_source_file:
        destination_path = output_dir.path + "/" + package.package_path + "/build.rs"
        action_inputs.append(package.build_script_source_file)
        shell_commands.append("cp -fL {} {}".format(package.build_script_source_file.path, destination_path))
    return action_inputs, shell_commands

def _generate_copy_data_files_commands(package, output_dir):
    shell_commands = []
    action_inputs = []
    source_dir = getattr(package, "source_package_path", package.package_path)

    if hasattr(package, "build_script_data_files"):
        for file in package.build_script_data_files:
            if file.basename == "Cargo.toml":
                continue
            relative_path = file.short_path
            if relative_path.startswith(source_dir + "/"):
                relative_path = relative_path[len(source_dir) + 1:]
            destination_path = output_dir.path + "/" + package.package_path + "/" + relative_path

            action_inputs.append(file)
            shell_commands.append("mkdir -p $(dirname {0})\ncp -RfL {1} {0}".format(destination_path, file.path))
    return action_inputs, shell_commands

def _generate_commands_for_package(context, package, output_dir):
    all_action_inputs = []
    shell_commands = []

    toml_content_lines = _generate_cargo_toml_content_lines(package)
    toml_artifact = _create_merged_cargo_toml_artifact(context, package, toml_content_lines)
    all_action_inputs.append(toml_artifact)
    shell_commands.append(_generate_copy_toml_command(package, output_dir, toml_artifact))

    source_inputs, source_commands = _generate_copy_source_files_commands(package, output_dir)
    all_action_inputs.extend(source_inputs)
    shell_commands.extend(source_commands)

    bs_inputs, bs_commands = _generate_copy_build_script_commands(package, output_dir)
    all_action_inputs.extend(bs_inputs)
    shell_commands.extend(bs_commands)

    data_inputs, data_commands = _generate_copy_data_files_commands(package, output_dir)
    all_action_inputs.extend(data_inputs)
    shell_commands.extend(data_commands)

    return all_action_inputs, shell_commands

def _create_workspace_toml_artifact(context, output_dir, workspace_members):
    lines = ["[workspace]", 'resolver = "2"', "members = ["]
    for member in workspace_members:
        lines.append('    "{}",'.format(member))
    lines.append("]")

    artifact = context.actions.declare_file("intermediate_manifests/root/Cargo.toml")
    context.actions.write(artifact, "\n".join(lines))

    shell_command = "cp -fL {} {}/Cargo.toml".format(artifact.path, output_dir.path)
    return artifact, shell_command

def _export_cargo_package_impl(context):
    cargo_gen_info = context.attr.target[CargoGenInfo]
    output_dir = context.actions.declare_directory("cargo_package")

    all_action_inputs = []
    shell_commands = ["mkdir -p " + output_dir.path]
    workspace_members = []
    processed_paths = {}

    for package in cargo_gen_info.transitive_packages.to_list():
        if package.is_external or package.package_path in processed_paths:
            continue
        processed_paths[package.package_path] = True
        workspace_members.append(package.package_path)

        package_inputs, package_commands = _generate_commands_for_package(context, package, output_dir)
        all_action_inputs.extend(package_inputs)
        shell_commands.extend(package_commands)

    workspace_artifact, workspace_command = _create_workspace_toml_artifact(context, output_dir, workspace_members)
    all_action_inputs.append(workspace_artifact)
    shell_commands.append(workspace_command)

    linker_script = context.actions.declare_file("linker.sh")
    context.actions.write(linker_script, "\n".join(shell_commands))

    context.actions.run_shell(
        inputs = all_action_inputs + [linker_script],
        outputs = [output_dir],
        command = "bash {}".format(linker_script.path),
        mnemonic = "PopulateCargoDir",
    )

    return [DefaultInfo(files = depset([output_dir]))]

export_cargo_package = rule(
    implementation = _export_cargo_package_impl,
    attrs = {
        "target": attr.label(aspects = [cargo_aspect], mandatory = True),
    },
)
