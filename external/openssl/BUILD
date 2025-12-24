"""An openssl build file based on a snippet found in the github issue:
https://github.com/bazel-contrib/rules_foreign_cc/issues/337

Note that the $(PERL) "make variable" (https://docs.bazel.build/versions/main/be/make-variables.html)
is populated by the perl toolchain provided by rules_perl.
"""

load("@rules_foreign_cc//foreign_cc:defs.bzl", "configure_make", "configure_make_variant")

# Read https://wiki.openssl.org/index.php/Compilation_and_Installation

CONFIGURE_OPTIONS = [
    "no-comp",
    "no-idea",
    "no-weak-ssl-ciphers",
    "no-tests",
]

LIB_NAME = "openssl"

MAKE_TARGETS = [
    "build_libs",
    "install_dev",
]

config_setting(
    name = "msvc_compiler",
    flag_values = {
        "@bazel_tools//tools/cpp:compiler": "msvc-cl",
    },
    visibility = ["//visibility:public"],
)

alias(
    name = "openssl",
    actual = select({
        ":msvc_compiler": "openssl_msvc",
        "//conditions:default": "openssl_default",
    }),
    visibility = ["//visibility:public"],
)

configure_make_variant(
    name = "openssl_msvc",
    build_data = [
        "@nasm//:nasm",
    ],
    configure_command = "Configure",
    configure_in_place = True,
    configure_options = CONFIGURE_OPTIONS + [
        "--libdir=lib",
        # Unset Microsoft Assembler (MASM) flags set by built-in MSVC toolchain,
        # as NASM is unsed to build OpenSSL rather than MASM
        "CC=\"cl\"",
        "ASFLAGS=\" \"",
        "CFLAGS=\"/W3 /wd4090 /nologo /O2 /Zi /MP\"",
        "LDFLAGS=\"/nologo /debug /Zi\"",
        "ARFLAGS=\"/nologo\"",
        "LD=\"link\"",
    ],
    configure_prefix = "$$PERL",
    env = {
        "PATH": "$$(dirname $(execpath @nasm//:nasm))",
        "PERL": "$$EXT_BUILD_ROOT$$/$(PERL)",
    },
    lib_name = LIB_NAME,
    lib_source = "@openssl//:all_srcs",
    out_static_libs = [
        "libssl.lib",
        "libcrypto.lib",
    ],
    postfix_script = """
        mkdir -p $$INSTALLDIR/lib/pkgconfig

        cat <<EOF > $$INSTALLDIR/lib/pkgconfig/libcrypto.pc
prefix=$$INSTALLDIR
exec_prefix=\\$${prefix}
libdir=\\$${prefix}/lib
includedir=\\$${prefix}/include

Name: OpenSSL-libcrypto
Description: OpenSSL cryptography library
Version: 3.0.0
Libs: -L\\$${libdir} -llibcrypto
Libs.private: -lcrypt32 -lws2_32 -ladvapi32 -luser32
Cflags: -I\\$${includedir}
EOF

        cat <<EOF > $$INSTALLDIR/lib/pkgconfig/libssl.pc
prefix=$$INSTALLDIR
exec_prefix=\\$${prefix}
libdir=\\$${prefix}/lib
includedir=\\$${prefix}/include

Name: OpenSSL-libssl
Description: Secure Sockets Layer and cryptography libraries
Version: 3.0.0
Requires: libcrypto
Libs: -L\\$${libdir} -llibssl
Cflags: -I\\$${includedir}
EOF

        cat <<EOF > $$INSTALLDIR/lib/pkgconfig/openssl.pc
prefix=$$INSTALLDIR
exec_prefix=\\$${prefix}
libdir=\\$${prefix}/lib
includedir=\\$${prefix}/include

Name: OpenSSL
Description: Secure Sockets Layer and cryptography libraries
Version: 3.0.0
Requires: libssl libcrypto
EOF
    """,
    targets = MAKE_TARGETS,
    toolchain = "@rules_foreign_cc//toolchains:preinstalled_nmake_toolchain",
    toolchains = ["@rules_perl//:current_toolchain"],
)

configure_make(
    name = "openssl_default",
    configure_command = "config",
    configure_in_place = True,
    configure_options = CONFIGURE_OPTIONS,
    env = select({
        "@platforms//os:macos": {
            "AR": "",
            "PERL": "$$EXT_BUILD_ROOT$$/$(PERL)",
        },
        "//conditions:default": {
            "PERL": "$$EXT_BUILD_ROOT$$/$(PERL)",
        },
    }),
    lib_name = LIB_NAME,
    lib_source = "@openssl//:all_srcs",
    # Note that for Linux builds, libssl must come before libcrypto on the linker command-line.
    # As such, libssl must be listed before libcrypto
    out_lib_dir = "lib64",
    out_static_libs = [
        "libssl.a",
        "libcrypto.a",
    ],
    targets = MAKE_TARGETS,
    toolchains = ["@rules_perl//:current_toolchain"],
)

filegroup(
    name = "gen_dir",
    srcs = [":openssl"],
    output_group = "gen_dir",
    visibility = ["//visibility:public"],
)
