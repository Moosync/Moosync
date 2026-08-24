package(default_visibility = ["//visibility:public"])

sh_binary(
    name = "format",
    srcs = ["format.sh"],
    args = [
        "$(rlocationpath @rules_rust//tools/upstream_wrapper:rustfmt)",
        "$(rlocationpath @crates//:slint-lsp__slint-lsp)",
    ],
    data = [
        "@rules_rust//tools/upstream_wrapper:rustfmt",
        "@crates//:slint-lsp__slint-lsp",
    ],
    deps = ["@bazel_tools//tools/bash/runfiles"],
)

py_binary(
    name = "check_instrument",
    srcs = ["check_instrument.py"],
    main = "check_instrument.py",
)

genrule(
    name = "gen_extract_translations",
    outs = ["extract_translations.sh"],
    cmd = """cat << 'EOF' > $@
#!/usr/bin/env bash
set -euo pipefail
EXTRACTOR="$$(pwd)/$(rootpath @bindeps//:slint-tr-extractor__slint-tr-extractor)"
cd "$${BUILD_WORKSPACE_DIRECTORY:-$$(pwd)}"
if [ $$# -gt 0 ]; then
  exec "$$EXTRACTOR" --no-default-translation-context -o ui/slint/locales/slint_app.pot "$$@"
else
  find ui/slint/src -name "*.slint" | sort | xargs "$$EXTRACTOR" --no-default-translation-context -o ui/slint/locales/slint_app.pot
fi
EOF
""",
    tools = ["@bindeps//:slint-tr-extractor__slint-tr-extractor"],
)

sh_binary(
    name = "extract_translations",
    srcs = [":gen_extract_translations"],
    data = ["@bindeps//:slint-tr-extractor__slint-tr-extractor"],
)

