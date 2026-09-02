#!/usr/bin/env python3
import os
import re
import sys

# Regex to match function declarations
# Handles: pub fn, fn, async fn, pub async fn, pub(crate) fn, etc.
FN_RE = re.compile(r'^\s*(?:pub\s+(?:\([^)]+\)\s+)?)?(?:async\s+)?(?:const\s+)?fn\s+([a-zA-Z0-9_]+)')

# Regex to match async blocks / async closures (not async fn)
ASYNC_BLOCK_RE = re.compile(r'\basync\b(?:\s+move)?(?:\s*\|[^|]*\|)?\s*\{')

def get_all_rs_files(project_root):
    rs_files = []
    # Check core/ and ui/slint/src/
    search_dirs = [
        os.path.join(project_root, "core"),
        os.path.join(project_root, "ui", "slint", "src")
    ]
    for search_dir in search_dirs:
        if not os.path.exists(search_dir):
            continue
        for root, _, files in os.walk(search_dir):
            # Exclude proc macro crates and common test/bench patterns
            if "plugin_macro" in root or "theme_macro" in root or "benches" in root:
                continue
            for file in files:
                if file.endswith(".rs"):
                    # Skip test and build files
                    if "test" in file or "bench" in file or file == "build.rs":
                        continue
                    rs_files.append(os.path.relpath(os.path.join(root, file), project_root))
    return rs_files

def check_async_blocks(content, lines):
    """
    Finds all async blocks/closures (e.g. `async move { ... }` or `async { ... }`)
    and verifies that `.in_current_span()` or `.instrument(...)` is chained on them.
    """
    errors = []
    
    # Simple scanner over content
    # Look for occurrences of 'async' not followed by 'fn '
    idx = 0
    length = len(content)
    
    while idx < length:
        # Check for 'async' keyword boundary
        match = re.search(r'\basync\b', content[idx:])
        if not match:
            break
            
        async_start = idx + match.start()
        idx = async_start + 5
        
        # Check what follows 'async'
        after_async = content[idx:].lstrip()
        if after_async.startswith("fn ") or after_async.startswith("fn\t") or after_async.startswith("fn\n"):
            continue
            
        # Must match `async move {` or `async {` or `async |...| {` or `async move |...| {`
        block_match = ASYNC_BLOCK_RE.match(content[async_start:])
        if not block_match:
            continue
            
        # Find the opening brace of this async block
        brace_pos = async_start + block_match.end() - 1
        
        # Calculate line number (1-indexed)
        line_num = content[:async_start].count('\n') + 1
        
        # Match the balanced closing brace
        depth = 0
        in_string = False
        in_char = False
        in_line_comment = False
        in_block_comment = False
        pos = brace_pos
        closing_brace_pos = -1
        
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
                    pos += 1  # Skip escaped char
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
                        closing_brace_pos = pos
                        break
            pos += 1
            
        if closing_brace_pos == -1:
            continue
            
        # Inspect what comes immediately after the closing brace (skipping whitespace/comments)
        after_close = content[closing_brace_pos + 1:].lstrip()
        # Clean leading comments
        while after_close.startswith("//") or after_close.startswith("/*"):
            if after_close.startswith("//"):
                newline_idx = after_close.find("\n")
                if newline_idx == -1:
                    after_close = ""
                    break
                after_close = after_close[newline_idx + 1:].lstrip()
            elif after_close.startswith("/*"):
                end_comment = after_close.find("*/")
                if end_comment == -1:
                    after_close = ""
                    break
        has_instrument = bool(
            re.match(r'^\.(in_current_span\s*\(\s*\)|instrument\s*\()', after_close)
        )
        
        if not has_instrument:
            snippet = lines[line_num - 1].strip() if line_num - 1 < len(lines) else "async block"
            errors.append((line_num, "async block/closure", snippet, "Missing '.in_current_span()' or '.instrument(...)' on async block"))
            
        idx = closing_brace_pos + 1
        
    return errors

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
        
        # Track if we are inside a test module or test function
        if "mod test" in striped or "mod tests" in striped:
            in_test_mod = True
        
        # Track trait definitions (trait methods without bodies shouldn't be instrumented)
        if striped.startswith("pub trait ") or striped.startswith("trait "):
            in_trait_def = True
            
        # Trivial brace counting for block exits
        if "{" in line:
            brace_count += line.count("{")
        if "}" in line:
            brace_count -= line.count("}")
            if brace_count <= 0:
                in_test_mod = False
                in_trait_def = False
                brace_count = 0

        # Don't require instrumentation inside tests or trait definitions
        if in_test_mod or in_trait_def:
            continue

        match = FN_RE.match(line)
        if match:
            fn_name = match.group(1)
            
            # Skip common built-in trait method names that can't easily be instrumented
            # or shouldn't be (like format, main, test, etc.)
            if fn_name in ["main", "default", "from", "into", "try_from", "fmt", "clone", "drop"]:
                continue

            # Look backwards up to 5 lines for tracing::instrument
            instrumented = False
            for lookback in range(1, 6):
                prev_idx = idx - lookback
                if prev_idx < 0:
                    break
                prev_line = lines[prev_idx].strip()
                # If we encounter another statement/expression, stop looking
                if prev_line.endswith(";") or prev_line.endswith("}") or (prev_line.startswith("fn ") and not prev_line.endswith("{")):
                    break
                if "#[tracing::instrument" in prev_line or "#[instrument" in prev_line:
                    instrumented = True
                    break
            
            if not instrumented:
                # Find if the function declaration ends with ; (meaning it's an FFI declaration or trait decl)
                full_decl = striped
                f_idx = idx
                while ")" not in lines[f_idx] and f_idx + 1 < len(lines):
                    f_idx += 1
                    full_decl += " " + lines[f_idx].strip()
                
                # Check up to 5 lines ahead for ; or {
                has_body = True
                for lookahead in range(0, 5):
                    if f_idx + lookahead < len(lines):
                        ahead_line = lines[f_idx + lookahead].strip()
                        if ";" in ahead_line:
                            has_body = False
                            break
                        if "{" in ahead_line:
                            break
                
                if has_body:
                    missing.append((idx + 1, f"Function '{fn_name}'", striped, "Function is missing #[tracing::instrument]"))

    # Check for async blocks without .in_current_span()
    async_errors = check_async_blocks(content, lines)
    missing.extend(async_errors)
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

