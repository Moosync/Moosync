package(default_visibility = ["//visibility:public"])

sh_binary(
    name = "format",
    srcs = ["format.sh"],
    args = [
        "$(rlocationpath @rules_rust//tools/upstream_wrapper:rustfmt)",
        "$(rlocationpath @crates//:slint-lsp__slint-lsp)",
        "$(rlocationpath @buildifier_prebuilt//:buildifier)",
        "$(rlocationpath @rules_buf_toolchains//:buf)",
    ],
    data = [
        "@buildifier_prebuilt//:buildifier",
        "@crates//:slint-lsp__slint-lsp",
        "@rules_buf_toolchains//:buf",
        "@rules_rust//tools/upstream_wrapper:rustfmt",
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
mkdir -p ui/slint/locales/en_US/LC_MESSAGES
PO_FILE="ui/slint/locales/en_US/LC_MESSAGES/slint_app.po"
JOIN_FLAG=""
if [ -f "$$PO_FILE" ]; then
  JOIN_FLAG="-j"
fi

if [ $$# -gt 0 ]; then
  "$$EXTRACTOR" --package-name slint_app --no-default-translation-context $$JOIN_FLAG -o "$$PO_FILE" "$$@"
else
  mapfile -t FILES < <(find ui/slint/src -name "*.slint" | sort)
  "$$EXTRACTOR" --package-name slint_app --no-default-translation-context $$JOIN_FLAG -o "$$PO_FILE" "$${FILES[@]}"
fi

TMP_FILE="$${PO_FILE}.tmp"
awk '
/^#: / {
  for (i = 2; i <= NF; i++) {
    token = $$i
    if (index(token, "src/") == 1) {
      token = "ui/slint/" token
    }
    if (!seen[token]++) {
      tokens[++n] = token
    }
  }
  next
}
{
  if (n > 0) {
    printf "#:"
    for (i = 1; i <= n; i++) {
      printf " %s", tokens[i]
    }
    printf "\\n"
    delete seen
    delete tokens
    n = 0
  }
  print
}
END {
  if (n > 0) {
    printf "#:"
    for (i = 1; i <= n; i++) {
      printf " %s", tokens[i]
    }
    printf "\\n"
  }
}
' "$$PO_FILE" > "$$TMP_FILE"
mv "$$TMP_FILE" "$$PO_FILE"
EOF
""",
    tools = ["@bindeps//:slint-tr-extractor__slint-tr-extractor"],
)

sh_binary(
    name = "extract_translations",
    srcs = [":gen_extract_translations"],
    data = ["@bindeps//:slint-tr-extractor__slint-tr-extractor"],
)
