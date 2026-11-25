#!/usr/bin/env python3
"""Verify that MODULE.bazel uses git_override instead of local overrides."""

import ast
import sys
from pathlib import Path

REQUIRED_GIT_OVERRIDE_MODULES = {"moosync", "extensions_sdk"}
FORBIDDEN_OVERRIDE_FUNCS = {"local_path_override", "local_repository"}


def _kwarg_str(call: ast.Call, name: str) -> str | None:
    for kw in call.keywords:
        if kw.arg == name and isinstance(kw.value, ast.Constant) and isinstance(kw.value.value, str):
            return kw.value.value
    return None


def check_module_bazel(content: str, label: str = "MODULE.bazel") -> list[str]:
    """Return a list of error messages if MODULE.bazel has local overrides or missing git_override."""
    try:
        tree = ast.parse(content, filename=label)
    except SyntaxError as exc:
        return [f"{label}:{exc.lineno}: syntax error: {exc.msg}"]

    errors: list[str] = []
    declared_deps: set[str] = set()
    git_overrides: dict[str, ast.Call] = {}

    for node in tree.body:
        if not isinstance(node, ast.Expr) or not isinstance(node.value, ast.Call):
            continue
        call = node.value
        if not isinstance(call.func, ast.Name):
            continue

        func_name = call.func.id
        if func_name in FORBIDDEN_OVERRIDE_FUNCS:
            mod = _kwarg_str(call, "module_name") or _kwarg_str(call, "name") or "<unknown>"
            path = _kwarg_str(call, "path") or "<unknown>"
            errors.append(
                f"{label}:{node.lineno}: forbidden {func_name}(module_name={mod!r}, path={path!r}); use git_override instead"
            )
        elif func_name == "bazel_dep":
            mod = _kwarg_str(call, "name")
            if mod:
                declared_deps.add(mod)
        elif func_name == "git_override":
            mod = _kwarg_str(call, "module_name")
            if mod:
                git_overrides[mod] = call

    for mod in sorted(declared_deps & REQUIRED_GIT_OVERRIDE_MODULES):
        call = git_overrides.get(mod)
        if call is None:
            errors.append(f"{label}: bazel_dep({mod!r}) must have a git_override")
            continue
        remote = _kwarg_str(call, "remote")
        if not remote or not (remote.startswith("https://") or remote.startswith("http://") or remote.startswith("ssh://") or remote.startswith("git@")):
            errors.append(f"{label}:{call.lineno}: git_override({mod!r}) must specify a valid remote URL")

    return errors


def main() -> int:
    args = sys.argv[1:]
    if args == ["--stdin"]:
        content = sys.stdin.read()
        errors = check_module_bazel(content, "MODULE.bazel")
        if errors:
            for err in errors:
                print(err, file=sys.stderr)
            return 1
        sys.stdout.write(content)
        return 0

    if not args:
        import os
        ws = os.environ.get("BUILD_WORKSPACE_DIRECTORY")
        base = Path(ws) if ws else Path.cwd()
        targets = [base / "MODULE.bazel"]
    else:
        targets = [Path(p) for p in args]

    all_errors: list[str] = []
    for path in targets:
        if not path.is_file():
            all_errors.append(f"{path}: file not found")
            continue
        all_errors.extend(check_module_bazel(path.read_text(encoding="utf-8"), str(path)))

    if all_errors:
        for err in all_errors:
            print(err, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
