#!/usr/bin/env python3
import os
import re
import sys

# Regex to match function declarations
# Handles: pub fn, fn, async fn, pub async fn, pub(crate) fn, etc.
FN_RE = re.compile(r'^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:const\s+)?fn\s+(\w+)')

# Regex to match async blocks / async closures (not async fn)
ASYNC_BLOCK_RE = re.compile(r'\basync\b(?:\s+move)?(?:\s*\|[^|]*\|)?\s*\{')

try:
    from tools.instrument_common import is_valid_rs_file, get_all_rs_files, IGNORE_DIRECTIVE_RE
except ImportError:
    from instrument_common import is_valid_rs_file, get_all_rs_files, IGNORE_DIRECTIVE_RE

def skip_line_comment(content, pos, length):
    newline_pos = content.find('\n', pos)
    return newline_pos if newline_pos != -1 else length

def skip_block_comment(content, pos, length):
    end_pos = content.find('*/', pos)
    return end_pos + 1 if end_pos != -1 else length

def skip_quoted_literal(content, pos, length, quote_char):
    pos += 1
    while pos < length:
        if content[pos] == '\\':
            pos += 2
        elif content[pos] == quote_char:
            return pos
        else:
            pos += 1
    return length

def find_matching_brace(content, brace_pos, length):
    depth = 0
    pos = brace_pos
    while pos < length:
        ch = content[pos]
        if ch == '/' and pos + 1 < length:
            if content[pos + 1] == '/':
                pos = skip_line_comment(content, pos + 2, length)
            elif content[pos + 1] == '*':
                pos = skip_block_comment(content, pos + 2, length)
        elif ch in ('"', "'"):
            pos = skip_quoted_literal(content, pos, length, ch)
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

def should_skip_scope(line, scope_state):
    striped = line.strip()
    if "mod test" in striped or "mod tests" in striped:
        scope_state["in_test"] = True
    if striped.startswith(("pub trait ", "trait ")):
        scope_state["in_trait"] = True

    if "{" in line:
        scope_state["braces"] += line.count("{")
    if "}" in line:
        scope_state["braces"] -= line.count("}")
        if scope_state["braces"] <= 0:
            scope_state["in_test"] = False
            scope_state["in_trait"] = False
            scope_state["braces"] = 0

    return scope_state["in_test"] or scope_state["in_trait"]

def check_function_instrumentation(lines, idx, line):
    match = FN_RE.match(line)
    if not match:
        return None
    fn_name = match.group(1)
    if fn_name in ["main", "default", "from", "into", "try_from", "fmt", "clone", "drop"]:
        return None

    striped = line.strip()
    ignored, instrumented, invalid_err = inspect_preceding_attributes(lines, idx, fn_name, striped)
    if invalid_err:
        return invalid_err
    if ignored or instrumented:
        return None
    if has_function_body(lines, idx):
        return (idx + 1, f"Function '{fn_name}'", striped, "Function is missing #[tracing::instrument]")
    return None

def check_file(filepath):
    if not os.path.exists(filepath):
        return []

    with open(filepath, 'r') as f:
        content = f.read()

    lines = content.splitlines()
    missing = []
    scope_state = {"in_test": False, "in_trait": False, "braces": 0}

    for idx, line in enumerate(lines):
        if should_skip_scope(line, scope_state):
            continue
        err = check_function_instrumentation(lines, idx, line)
        if err:
            missing.append(err)

    missing.extend(check_async_blocks(content, lines))
    missing.sort(key=lambda x: x[0])
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

