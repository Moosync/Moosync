load("@bazel_tools//tools/cpp:cc_toolchain_config_lib.bzl", "tool_path")
load("@rules_cc//cc/toolchains:cc_toolchain_config_info.bzl", "CcToolchainConfigInfo")

def _impl(ctx):
    tool_paths = [
        tool_path(name = "gcc", path = "bin/x86_64-w64-mingw32-gcc"),
        tool_path(name = "g++", path = "bin/x86_64-w64-mingw32-g++"),
        tool_path(name = "ld", path = "bin/x86_64-w64-mingw32-ld"),
        tool_path(name = "ar", path = "bin/x86_64-w64-mingw32-ar"),
        tool_path(name = "cpp", path = "bin/x86_64-w64-mingw32-cpp"),
        tool_path(name = "gcov", path = "bin/x86_64-w64-mingw32-gcov"),
        tool_path(name = "nm", path = "bin/x86_64-w64-mingw32-nm"),
        tool_path(name = "objdump", path = "bin/x86_64-w64-mingw32-objdump"),
        tool_path(name = "strip", path = "bin/x86_64-w64-mingw32-strip"),
        tool_path(name = "dwp", path = "bin/x86_64-w64-mingw32-elfedit"),
    ]

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
        cxx_builtin_include_directories = [
            "%package(//)%/include",
            "%package(//)%/x86_64-w64-mingw32/include",
            "%package(//)%/lib/gcc/x86_64-w64-mingw32/11.2.1/include",
            "%package(//)%/lib/gcc/x86_64-w64-mingw32/11.2.1/include-fixed",
        ],
    )

mingw_cc_toolchain_config = rule(
    implementation = _impl,
    attrs = {},
    provides = [CcToolchainConfigInfo],
)
