#!/usr/bin/env python3
import os
import re
import sys

# Regex to match function declarations
# Handles: pub fn, fn, async fn, pub async fn, pub(crate) fn, etc.
FN_RE = re.compile(r'^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:const\s+)?fn\s+(\w+)')

# Regex to match async blocks / async closures (not async fn)
ASYNC_BLOCK_RE = re.compile(r'\basync\b(?:\s+move)?(?:\s*\|[^|]*\|)?\s*\{')

# Regex to match ignore directive with a non-empty reason
# Example: // moosync::no_instrument(reason = "hot loop: called per audio sample")
IGNORE_DIRECTIVE_RE = re.compile(r'^\s*//\s*moosync::no_instrument\s*\(\s*reason\s*=\s*["\']([^"\']+)["\']\s*\)')

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

def find_matching_brace(content, brace_pos, length):
    depth = 0
    in_string = False
    in_char = False
    in_line_comment = False
    in_block_comment = False
    pos = brace_pos

    while pos < length:
        ch = content[pos]
        if in_line_comment:
            if ch == '\n':
                in_line_comment = False
        elif in_block_comment:
            if ch == '*' and pos + 1 < length and content[pos + 1] == '/':
                in_block_comment = False
                pos += 1
        elif in_string:
            if ch == '\\':
                pos += 1
            elif ch == '"':
                in_string = False
        elif in_char:
            if ch == '\\':
                pos += 1
            elif ch == "'":
                in_char = False
        else:
            if ch == '/' and pos + 1 < length:
                if content[pos + 1] == '/':
                    in_line_comment = True
                    pos += 1
                elif content[pos + 1] == '*':
                    in_block_comment = True
                    pos += 1
            elif ch == '"':
                in_string = True
            elif ch == "'":
                in_char = True
            elif ch == '{':
                depth += 1
            elif ch == '}':
                depth -= 1
                if depth == 0:
                    return pos
        pos += 1
    return -1

def clean_leading_comments(text):
    after_close = text.lstrip()
    while after_close.startswith(("//", "/*")):
        if after_close.startswith("//"):
            newline_idx = after_close.find("\n")
            if newline_idx == -1:
                return ""
            after_close = after_close[newline_idx + 1:].lstrip()
        elif after_close.startswith("/*"):
            end_comment = after_close.find("*/")
            if end_comment == -1:
                return ""
            after_close = after_close[end_comment + 2:].lstrip()
    return after_close

def check_async_blocks(content, lines):
    """
    Finds all async blocks/closures (e.g. `async move { ... }` or `async { ... }`)
    and verifies that `.in_current_span()` or `.instrument(...)` is chained on them.
    """
    errors = []
    idx = 0
    length = len(content)

    while idx < length:
        match = re.search(r'\basync\b', content[idx:])
        if not match:
            break

        async_start = idx + match.start()
        idx = async_start + 5

        after_async = content[idx:].lstrip()
        if after_async.startswith(("fn ", "fn\t", "fn\n")):
            continue

        block_match = ASYNC_BLOCK_RE.match(content[async_start:])
        if not block_match:
            continue

        brace_pos = async_start + block_match.end() - 1
        line_num = content[:async_start].count('\n') + 1

        closing_brace_pos = find_matching_brace(content, brace_pos, length)
        if closing_brace_pos == -1:
            continue

        after_close = clean_leading_comments(content[closing_brace_pos + 1:])
        has_instrument = bool(
            re.match(r'^\.(in_current_span\s*\(\s*\)|instrument\s*\()', after_close)
        )

        if not has_instrument:
            snippet = lines[line_num - 1].strip() if line_num - 1 < len(lines) else "async block"
            errors.append((line_num, "async block/closure", snippet, "Missing '.in_current_span()' or '.instrument(...)' on async block"))

        idx = closing_brace_pos + 1

    return errors

def inspect_preceding_attributes(lines, idx, fn_name, striped):
    instrumented = False
    comment_lines = []
    for lookback in range(1, 8):
        prev_idx = idx - lookback
        if prev_idx < 0:
            break
        prev_line = lines[prev_idx].strip()
        if prev_line.startswith("//"):
            comment_lines.append(prev_line.lstrip("/").strip())
        elif "#[tracing::instrument" in prev_line or "#[instrument" in prev_line:
            instrumented = True
            break
        elif prev_line.startswith("#["):
            continue
        elif prev_line.endswith(";") or prev_line.endswith("}") or (prev_line.startswith("fn ") and not prev_line.endswith("{")):
            break

    joined_comments = " ".join(reversed(comment_lines))
    if "moosync::no_instrument" in joined_comments:
        directive_match = re.search(r'moosync::no_instrument\s*\(\s*reason\s*=\s*["\']([^"\']+)["\']\s*\)', joined_comments)
        if directive_match and directive_match.group(1).strip():
            return True, False, None
        invalid_err = (
            idx + 1,
            f"Function '{fn_name}'",
            striped,
            "Directive '// moosync::no_instrument' must specify a non-empty reason: // moosync::no_instrument(reason = \"...\")"
        )
        return False, False, invalid_err

    return False, instrumented, None

def has_function_body(lines, idx):
    f_idx = idx
    while ")" not in lines[f_idx] and f_idx + 1 < len(lines):
        f_idx += 1

    for lookahead in range(0, 5):
        if f_idx + lookahead < len(lines):
            ahead_line = lines[f_idx + lookahead].strip()
            if ";" in ahead_line:
                return False
            if "{" in ahead_line:
                return True
    return True

def check_file(filepath):
    if not os.path.exists(filepath):
        return []

    with open(filepath, 'r') as f:
        content = f.read()

    lines = content.splitlines()
    missing = []
    in_test_mod = False
    in_trait_def = False
    brace_count = 0

    for idx, line in enumerate(lines):
        striped = line.strip()

        if "mod test" in striped or "mod tests" in striped:
            in_test_mod = True

        if striped.startswith(("pub trait ", "trait ")):
            in_trait_def = True

        if "{" in line:
            brace_count += line.count("{")
        if "}" in line:
            brace_count -= line.count("}")
            if brace_count <= 0:
                in_test_mod = False
                in_trait_def = False
                brace_count = 0

        if in_test_mod or in_trait_def:
            continue

        match = FN_RE.match(line)
        if match:
            fn_name = match.group(1)
            if fn_name in ["main", "default", "from", "into", "try_from", "fmt", "clone", "drop"]:
                continue

            ignored, instrumented, invalid_err = inspect_preceding_attributes(lines, idx, fn_name, striped)
            if invalid_err:
                missing.append(invalid_err)
                continue

            if ignored or instrumented:
                continue

            if has_function_body(lines, idx):
                missing.append((idx + 1, f"Function '{fn_name}'", striped, "Function is missing #[tracing::instrument]"))

    async_errors = check_async_blocks(content, lines)
    missing.extend(async_errors)
    missing.sort(key=lambda x: x[0])
    return missing

    return missing

def main():
    has_errors = False
    project_root = os.environ.get("BUILD_WORKSPACE_DIRECTORY", os.getcwd())
    
    print("Checking recursively all .rs files for #[tracing::instrument], .in_current_span(), and .instrument()...")
    target_files = get_all_rs_files(project_root)
    
    for rel_path in target_files:
        full_path = os.path.join(project_root, rel_path)
        missing = check_file(full_path)
        if missing:
            has_errors = True
            for line_num, item_name, decl, reason in missing:
                print(f"{rel_path}:{line_num}: {reason}")
                print(f"  Code: {decl}")
                
    if has_errors:
        print("\nError: Instrumentation checks failed.")
        sys.exit(1)
    else:
        print("\nSuccess: All check targets are properly instrumented.")
        sys.exit(0)

if __name__ == '__main__':
    main()

