load("@bazel_tools//tools/build_defs/cc:action_names.bzl", "ACTION_NAMES")
load("@bazel_tools//tools/cpp:cc_toolchain_config_lib.bzl", "feature", "flag_group", "flag_set", "tool_path")
load("@rules_cc//cc/common:cc_common.bzl", "cc_common")
load("@rules_cc//cc/toolchains:cc_toolchain_config_info.bzl", "CcToolchainConfigInfo")

def _impl(ctx):
    tool_paths = [
        tool_path(name = "gcc", path = "bin/x86_64-w64-mingw32-gcc"),
        tool_path(name = "g++", path = "bin/x86_64-w64-mingw32-g++"),
        tool_path(name = "ld", path = "bin/x86_64-w64-mingw32-ld"),
        tool_path(name = "ar", path = "bin/x86_64-w64-mingw32-ar"),
        tool_path(name = "cpp", path = "bin/x86_64-w64-mingw32-cpp"),
        tool_path(name = "gcov", path = "bin/llvm-profdata"),
        tool_path(name = "nm", path = "bin/x86_64-w64-mingw32-nm"),
        tool_path(name = "objdump", path = "bin/x86_64-w64-mingw32-objdump"),
        tool_path(name = "strip", path = "bin/x86_64-w64-mingw32-strip"),
        tool_path(name = "dwp", path = "bin/llvm-dwp"),
    ]

    default_linker_flags = feature(
        name = "default_linker_flags",
        enabled = True,
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.cpp_link_executable,
                    ACTION_NAMES.cpp_link_dynamic_library,
                    ACTION_NAMES.cpp_link_nodeps_dynamic_library,
                ],
                flag_groups = [
                    flag_group(
                        flags = [
                            "-Lexternal/mingw_compiler_files/x86_64-w64-mingw32/lib",
                        ],
                    ),
                ],
            ),
        ],
    )

    return cc_common.create_cc_toolchain_config_info(
        ctx = ctx,
        toolchain_identifier = "mingw-toolchain",
        host_system_name = "local",
        target_system_name = "x86_64-w64-mingw32",
        target_cpu = "x86_64",
        target_libc = "mingw",
        compiler = "mingw-gcc",
        abi_version = "mingw",
        abi_libc_version = "mingw",
        tool_paths = tool_paths,
        features = [default_linker_flags],
        cxx_builtin_include_directories = [
            "%package(//)%/include",
            "%package(//)%/include/c++/v1",
            "%package(//)%/x86_64-w64-mingw32/include",
        ],
    )

mingw_cc_toolchain_config = rule(
    implementation = _impl,
    attrs = {},
    provides = [CcToolchainConfigInfo],
)
