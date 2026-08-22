#!/usr/bin/env python3
import os
import re
import sys

# Regex to match function declarations
# Handles: pub fn, fn, async fn, pub async fn, pub(crate) fn, etc.
FN_RE = re.compile(r'^\s*(?:pub\s+(?:\([^)]+\)\s+)?)?(?:async\s+)?(?:const\s+)?fn\s+([a-zA-Z0-9_]+)')

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

def check_file(filepath):
    if not os.path.exists(filepath):
        return []

    with open(filepath, 'r') as f:
        lines = f.readlines()

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
                    missing.append((idx + 1, fn_name, striped))

    return missing

def main():
    has_errors = False
    project_root = os.environ.get("BUILD_WORKSPACE_DIRECTORY", os.getcwd())
    
    print("Checking recursively all .rs files for #[tracing::instrument]...")
    target_files = get_all_rs_files(project_root)
    
    for rel_path in target_files:
        full_path = os.path.join(project_root, rel_path)
        missing = check_file(full_path)
        if missing:
            has_errors = True
            print(f"\n{rel_path}:")
            for line_num, fn_name, decl in missing:
                print(f"  Line {line_num}: Function '{fn_name}' is missing instrumentation")
                print(f"    Code: {decl}")
                
    if has_errors:
        print("\nError: Some functions are missing #[tracing::instrument].")
        sys.exit(1)
    else:
        print("\nSuccess: All check targets are properly instrumented.")
        sys.exit(0)

if __name__ == '__main__':
    main()
