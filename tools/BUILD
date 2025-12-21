load("@rules_python//python:defs.bzl", "py_binary")

package(default_visibility = ["//visibility:public"])

# Wrapper for the HTML generator
py_binary(
    name = "replace_bin",
    srcs = ["replace.py"],
    main = "replace.py",
)

# Wrapper for the Directory assembler
py_binary(
    name = "assemble_bin",
    srcs = ["assemble.py"],
    main = "assemble.py",
)
