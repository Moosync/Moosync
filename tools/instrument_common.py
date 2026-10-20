import os
import re

FN_START_RE = re.compile(r'^(\s*)(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:const\s+)?fn\s+(\w+)')
IGNORE_DIRECTIVE_RE = re.compile(r'moosync::no_instrument\s*\(\s*reason\s*=\s*["\']([^"\']+)["\']\s*\)')

def is_valid_rs_file(root, file):
    if not file.endswith(".rs"):
        return False
    if "test" in file or "bench" in file or file == "build.rs":
        return False
    if "plugin_macro" in root or "theme_macro" in root or "benches" in root:
        return False
    return True

def get_all_rs_files(project_root):
    rs_files = []
    search_dirs = [
        os.path.join(project_root, "core"),
        os.path.join(project_root, "ui", "slint", "src")
    ]
    for search_dir in search_dirs:
        if not os.path.exists(search_dir):
            continue
        for root, _, files in os.walk(search_dir):
            for file in files:
                if is_valid_rs_file(root, file):
                    rs_files.append(os.path.relpath(os.path.join(root, file), project_root))
    return rs_files

def transform_rs_files(project_root, file_transformer):
    for rel_path in get_all_rs_files(project_root):
        path = os.path.join(project_root, rel_path)
        if not os.path.exists(path):
            continue
        with open(path, "r") as f:
            lines = f.readlines()
        new_lines = file_transformer(lines)
        if new_lines != lines:
            with open(path, "w") as f:
                f.writelines(new_lines)
